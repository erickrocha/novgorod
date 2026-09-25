use crate::{AppState, infrastructure::mercado_pago::MercadoPago};
use axum::{Json, body::Bytes, extract::{Extension, Path, Query, State}, http::{HeaderMap, StatusCode}};
use business::{domain::{enums::Role, marketplace::PurchaseError, user::User}, gateway::{payment_provider_gateway::{ChargeRequest, PaymentProviderGateway, ProviderResult, ProviderStatus}, purchase_gateway::PurchaseGateway}, sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, QueryFilter, Set, Statement, TransactionTrait}, use_cases::purchase_use_case::PurchaseUseCase};
use entity::{checkout_quote_entity, coupon_redemption_entity, credit_card_details_entity, order_status_history_entity, orders_entity, payment_entity, payment_transaction_entity, purchase_entity};
use serde::{Deserialize, Serialize};

type ApiError = (StatusCode, Json<serde_json::Value>);
fn bad(status: StatusCode, message: &str) -> ApiError { (status, Json(serde_json::json!({"message": message}))) }
fn unavailable() -> ApiError { bad(StatusCode::SERVICE_UNAVAILABLE, "Pagamento indisponível. Verifique a configuração do Mercado Pago.") }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentConfig { public_key: String }
pub async fn config(Extension(user): Extension<User>) -> Result<Json<PaymentConfig>, ApiError> {
    if user.role != Role::Customer { return Err(bad(StatusCode::FORBIDDEN, "Acesso negado")); }
    MercadoPago::configured().map_err(|_| unavailable())?;
    let public_key = std::env::var("MP_PUBLIC_KEY").map_err(|_| unavailable())?;
    if public_key.is_empty() { return Err(unavailable()); }
    Ok(Json(PaymentConfig { public_key }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubmitCard {
    token: String,
    payment_method_id: String,
    issuer_id: Option<String>,
    installments: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentState { purchase_id: i64, status: String, reference: Option<String> }

async fn owned(state: &AppState, user: &User, id: i64) -> Result<(purchase_entity::Model, payment_entity::Model), ApiError> {
    if user.role != Role::Customer { return Err(bad(StatusCode::FORBIDDEN, "Acesso negado")); }
    let use_case = PurchaseUseCase::new(PurchaseGateway::new(state.conn.as_ref().clone()));
    use_case.get(user, id).await.map_err(|e| match e {
        PurchaseError::NotFound | PurchaseError::Forbidden => bad(StatusCode::NOT_FOUND, "Compra não encontrada"),
        _ => bad(StatusCode::SERVICE_UNAVAILABLE, "Não foi possível consultar a compra"),
    })?;
    let purchase = purchase_entity::Entity::find_by_id(id).one(state.conn.as_ref()).await.map_err(|_| unavailable())?.ok_or_else(|| bad(StatusCode::NOT_FOUND, "Compra não encontrada"))?;
    let payment = payment_entity::Entity::find().filter(payment_entity::Column::PurchaseId.eq(id))
        .one(state.conn.as_ref()).await.map_err(|_| unavailable())?.ok_or_else(|| bad(StatusCode::NOT_FOUND, "Pagamento não encontrado"))?;
    Ok((purchase, payment))
}

fn state(purchase_id: i64, payment: &payment_entity::Model) -> PaymentState {
    PaymentState { purchase_id, status: payment.status.clone(), reference: payment.gateway_reference.clone() }
}

fn validate_provider(result: &ProviderResult, purchase: &purchase_entity::Model, payment: &payment_entity::Model, collector_id: i64) -> Result<(), ApiError> {
    if result.external_reference != format!("torg-{}-{}", purchase.id, payment.attempt_key.as_deref().unwrap_or_default()) || result.amount_cents != payment.amount_cents
        || result.currency != "BRL" || result.collector_id != collector_id || result.payment_type_id != "credit_card" {
        return Err(bad(StatusCode::BAD_GATEWAY, "Resposta de pagamento inconsistente"));
    }
    Ok(())
}

async fn apply_result(state: &AppState, purchase: &purchase_entity::Model, payment: &payment_entity::Model,
    result: ProviderResult, collector_id: i64) -> Result<PaymentState, ApiError> {
    validate_provider(&result, purchase, payment, collector_id)?;
    let tx = state.conn.begin().await.map_err(|_| unavailable())?;
    tx.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres,
        "SELECT id FROM payment WHERE id=$1 FOR UPDATE", [payment.id.into()])).await.map_err(|_| unavailable())?;
    let current = payment_entity::Entity::find_by_id(payment.id).one(&tx).await.map_err(|_| unavailable())?
        .ok_or_else(|| bad(StatusCode::NOT_FOUND, "Pagamento não encontrado"))?;
    if current.status == "captured" || (current.status == "failed" && result.status != ProviderStatus::Captured) {
        tx.commit().await.map_err(|_| unavailable())?;
        return Ok(self::state(purchase.id, &current));
    }
    if current.gateway_reference.as_deref().is_some_and(|r| r != result.reference) {
        return Err(bad(StatusCode::CONFLICT, "Referência de pagamento divergente"));
    }
    let status = match result.status { ProviderStatus::Captured => "captured", ProviderStatus::Failed => "failed",
        ProviderStatus::Authorized => "authorized", ProviderStatus::Pending => "pending" };
    let mut update: payment_entity::ActiveModel = current.into();
    update.status = Set(status.into());
    update.gateway_provider = Set(Some("mercado_pago".into()));
    update.gateway_reference = Set(Some(result.reference.clone()));
    update.attempt_error = Set(None);
    let saved = update.update(&tx).await.map_err(|_| unavailable())?;
    tx.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,
        "UPDATE payment_transaction SET status=$1,gateway_transaction_id=$2 WHERE payment_id=$3 AND idempotency_key=$4",
        [status.into(), result.reference.clone().into(), saved.id.into(), saved.attempt_key.clone().unwrap_or_default().into()])).await.map_err(|_| unavailable())?;
    if let Some(ref card_meta) = result.card {
        let existing_card = credit_card_details_entity::Entity::find()
            .filter(credit_card_details_entity::Column::PaymentId.eq(saved.id))
            .one(&tx).await.map_err(|_| unavailable())?;
        if let Some(card_row) = existing_card {
            let mut card_upd: credit_card_details_entity::ActiveModel = card_row.into();
            card_upd.cardholder_name = Set(card_meta.cardholder_name.clone());
            card_upd.brand = Set(card_meta.brand.clone());
            card_upd.last_four_digits = Set(card_meta.last_four_digits.clone());
            card_upd.expiration_month = Set(card_meta.expiration_month);
            card_upd.expiration_year = Set(card_meta.expiration_year);
            card_upd.update(&tx).await.map_err(|_| unavailable())?;
        } else {
            credit_card_details_entity::ActiveModel {
                payment_id: Set(saved.id),
                cardholder_name: Set(card_meta.cardholder_name.clone()),
                brand: Set(card_meta.brand.clone()),
                last_four_digits: Set(card_meta.last_four_digits.clone()),
                expiration_month: Set(card_meta.expiration_month),
                expiration_year: Set(card_meta.expiration_year),
                ..Default::default()
            }.insert(&tx).await.map_err(|_| unavailable())?;
        }
    }
    if matches!(result.status, ProviderStatus::Captured | ProviderStatus::Failed) {
        let (purchase_status, order_status, reservation_status) = if result.status == ProviderStatus::Captured {
            ("paid", "paid", "redeemed")
        } else { ("payment_failed", "payment_failed", "released") };
        tx.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,
            "UPDATE purchase SET status=$1,updated_at=CURRENT_TIMESTAMP WHERE id=$2", [purchase_status.into(), purchase.id.into()])).await.map_err(|_| unavailable())?;
        tx.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,
            "UPDATE orders SET status=$1,payment_status=$2,updated_at=CURRENT_TIMESTAMP WHERE purchase_id=$3",
            [order_status.into(), status.into(), purchase.id.into()])).await.map_err(|_| unavailable())?;
        tx.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,
            "UPDATE checkout_coupon_reservation SET status=$1 WHERE purchase_id=$2 AND status != 'redeemed'",
            [reservation_status.into(), purchase.id.into()])).await.map_err(|_| unavailable())?;
        let rows = orders_entity::Entity::find().filter(orders_entity::Column::PurchaseId.eq(purchase.id)).all(&tx).await.map_err(|_| unavailable())?;
        for order in &rows {
            order_status_history_entity::ActiveModel {
                tenant_id: Set(order.tenant_id),
                order_id: Set(order.id),
                from_status: Set(Some("pending_payment".into())),
                to_status: Set(order_status.into()),
                actor_type: Set("payment_gateway".into()),
                actor_id: Set(None),
                note: Set(Some(format!("mercado_pago:{}", result.reference))),
                ..Default::default()
            }.insert(&tx).await.map_err(|_| unavailable())?;
            if result.status == ProviderStatus::Captured {
                if let Some(coupon_id) = order.coupon_id {
                    coupon_redemption_entity::ActiveModel { tenant_id: Set(Some(order.tenant_id)), coupon_id: Set(coupon_id),
                        order_id: Set(order.id), customer_id: Set(purchase.customer_id), ..Default::default() }
                        .insert(&tx).await.map_err(|_| unavailable())?;
                }
            }
        }
    }
    tx.commit().await.map_err(|_| unavailable())?;
    Ok(self::state(purchase.id, &saved))
}

pub async fn submit(State(app): State<AppState>, Extension(user): Extension<User>, Path(id): Path<i64>,
    Json(card): Json<SubmitCard>) -> Result<Json<PaymentState>, ApiError> {
    let provider = MercadoPago::configured().map_err(|_| unavailable())?;
    if card.token.is_empty() || card.token.len() > 512 || card.payment_method_id.is_empty()
        || !card.payment_method_id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
        || card.installments < 1 || card.installments > 24 {
        return Err(bad(StatusCode::BAD_REQUEST, "Dados do cartão inválidos"));
    }
    let (purchase, payment) = owned(&app, &user, id).await?;
    if purchase.total_cents == 0 { return Ok(Json(state(id, &payment))); }
    if purchase.created_at + chrono::Duration::minutes(15) <= chrono::Utc::now().naive_utc() && payment.attempt_key.is_none() {
        return Err(bad(StatusCode::CONFLICT, "Prazo para pagamento expirado. Faça um novo orçamento."));
    }
    if checkout_quote_entity::Entity::find().filter(checkout_quote_entity::Column::PurchaseId.eq(id))
        .one(app.conn.as_ref()).await.map_err(|_| unavailable())?.is_none() {
        return Err(bad(StatusCode::CONFLICT, "Compra sem orçamento validado"));
    }
    let tx = app.conn.begin().await.map_err(|_| unavailable())?;
    tx.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres,
        "SELECT id FROM payment WHERE id=$1 FOR UPDATE", [payment.id.into()])).await.map_err(|_| unavailable())?;
    let current = payment_entity::Entity::find_by_id(payment.id).one(&tx).await.map_err(|_| unavailable())?
        .ok_or_else(|| bad(StatusCode::NOT_FOUND, "Pagamento não encontrado"))?;
    if current.status == "captured" || current.status == "authorized" || current.status == "pending" || (current.attempt_key.is_some() && current.status == "pending_provider") {
        tx.commit().await.map_err(|_| unavailable())?;
        return Ok(Json(state(id, &current)));
    }
    tx.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,
        "UPDATE checkout_coupon_reservation SET status='reserved' WHERE purchase_id=$1 AND status='released' AND expires_at > CURRENT_TIMESTAMP",
        [purchase.id.into()])).await.map_err(|_| unavailable())?;
    let key = uuid::Uuid::new_v4().to_string();
    let mut update: payment_entity::ActiveModel = current.into();
    update.attempt_key = Set(Some(key.clone()));
    update.attempt_started_at = Set(Some(chrono::Utc::now().naive_utc()));
    update.attempt_error = Set(None);
    update.status = Set("pending".into());
    update.installments = Set(card.installments);
    update.gateway_provider = Set(Some("mercado_pago".into()));
    update.gateway_reference = Set(None);
    let saved = update.update(&tx).await.map_err(|_| unavailable())?;
    payment_transaction_entity::ActiveModel { payment_id: Set(saved.id), operation: Set("charge".into()),
        status: Set("pending".into()), amount_cents: Set(saved.amount_cents), currency: Set("BRL".into()),
        idempotency_key: Set(key.clone()), gateway_provider: Set(Some("mercado_pago".into())),
        ..Default::default() }.insert(&tx).await.map_err(|_| unavailable())?;
    tx.commit().await.map_err(|_| unavailable())?;
    let charge = ChargeRequest { payment_id: saved.id, amount_cents: saved.amount_cents, currency: "BRL".into(),
        idempotency_key: key.clone(), provider_token: card.token, external_reference: format!("torg-{id}-{key}"),
        payer_email: purchase.customer_email.clone().unwrap_or_default(), payer_tax_id: purchase.customer_tax_id.clone(),
        payment_method_id: card.payment_method_id, issuer_id: card.issuer_id, installments: card.installments };
    match provider.charge(charge).await {
        Ok(result) => Ok(Json(apply_result(&app, &purchase, &saved, result, provider.collector_id).await?)),
        Err(e) => { log::warn!("Mercado Pago payment attempt requires reconciliation: {}", e.0);
            Ok(Json(state(id, &saved))) }
    }
}

pub async fn status(State(app): State<AppState>, Extension(user): Extension<User>, Path(id): Path<i64>) -> Result<Json<PaymentState>, ApiError> {
    let (purchase, payment) = owned(&app, &user, id).await?;
    if payment.status == "captured" || payment.status == "failed" || payment.status == "pending_provider" {
        return Ok(Json(state(id, &payment)));
    }
    let provider = MercadoPago::configured().map_err(|_| unavailable())?;
    let outcome = if let Some(reference) = &payment.gateway_reference { Some(provider.status(reference).await.map_err(|_| unavailable())?) }
        else { provider.search(&format!("torg-{id}-{}", payment.attempt_key.as_deref().unwrap_or_default())).await.map_err(|_| unavailable())? };
    if let Some(outcome) = outcome { Ok(Json(apply_result(&app, &purchase, &payment, outcome, provider.collector_id).await?)) }
    else { Ok(Json(state(id, &payment))) }
}

#[derive(Deserialize, Default)]
pub struct NotificationQuery {
    #[serde(rename = "data.id")] pub data_id: Option<String>,
    pub id: Option<String>,
}

pub fn verify_signature(secret: &str, signature: &str, request_id: &str, data_id: &str) -> Result<(), &'static str> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut ts = None;
    let mut v1 = None;
    for part in signature.split(',') {
        if let Some((k, v)) = part.trim().split_once('=') {
            match k { "ts" => ts = Some(v), "v1" => v1 = Some(v), _ => {} }
        }
    }
    let ts = ts.ok_or("Assinatura inválida")?;
    let expected = v1.ok_or("Assinatura inválida")?;
    let bytes = (0..expected.len()).step_by(2).map(|i| u8::from_str_radix(expected.get(i..i+2).unwrap_or(""), 16)).collect::<Result<Vec<_>,_>>()
        .map_err(|_| "Assinatura inválida")?;
    let manifest = format!("id:{data_id};request-id:{request_id};ts:{ts};");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| "Assinatura inválida")?;
    mac.update(manifest.as_bytes());
    mac.verify_slice(&bytes).map_err(|_| "Assinatura inválida")?;
    Ok(())
}

pub async fn webhook(State(app): State<AppState>, headers: HeaderMap, Query(query): Query<NotificationQuery>, body_bytes: Bytes) -> Result<StatusCode, ApiError> {
    let secret = std::env::var("MP_WEBHOOK_SECRET").map_err(|_| unavailable())?;
    let signature = headers.get("x-signature").and_then(|v| v.to_str().ok()).ok_or_else(|| bad(StatusCode::UNAUTHORIZED, "Assinatura ausente"))?;
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok()).ok_or_else(|| bad(StatusCode::UNAUTHORIZED, "Identificador ausente"))?;
    let mut data_id = query.data_id.or(query.id);
    if data_id.is_none() && !body_bytes.is_empty() {
        if let Ok(json_body) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
            data_id = json_body.get("data").and_then(|d| d.get("id")).and_then(|v| v.as_str().or_else(|| v.as_i64().map(|n| Box::leak(n.to_string().into_boxed_str()) as &str))).map(str::to_owned)
                .or_else(|| json_body.get("id").and_then(|v| v.as_str().or_else(|| v.as_i64().map(|n| Box::leak(n.to_string().into_boxed_str()) as &str))).map(str::to_owned));
        }
    }
    let data_id = data_id.ok_or_else(|| bad(StatusCode::UNAUTHORIZED, "Pagamento ausente"))?;
    if !data_id.bytes().all(|b| b.is_ascii_digit()) { return Err(bad(StatusCode::UNAUTHORIZED, "Pagamento inválido")); }
    verify_signature(&secret, signature, request_id, &data_id).map_err(|msg| bad(StatusCode::UNAUTHORIZED, msg))?;
    let provider = MercadoPago::configured().map_err(|_| unavailable())?;
    let outcome = provider.status(&data_id).await.map_err(|_| unavailable())?;
    let id = outcome.external_reference.strip_prefix("torg-").and_then(|s| s.split_once('-')).and_then(|(id, _)| id.parse::<i64>().ok());
    let Some(id) = id else { return Ok(StatusCode::OK); };
    let purchase = purchase_entity::Entity::find_by_id(id).one(app.conn.as_ref()).await.map_err(|_| unavailable())?;
    let Some(purchase) = purchase else { return Ok(StatusCode::OK); };
    let payment = payment_entity::Entity::find().filter(payment_entity::Column::PurchaseId.eq(id)).one(app.conn.as_ref()).await.map_err(|_| unavailable())?;
    let Some(payment) = payment else { return Ok(StatusCode::OK); };
    apply_result(&app, &purchase, &payment, outcome, provider.collector_id).await?;
    Ok(StatusCode::OK)
}

pub async fn reconcile_unresolved(state: &AppState) -> Result<usize, String> {
    let provider = match MercadoPago::configured() {
        Ok(p) => p,
        Err(_) => return Ok(0),
    };
    let payments = payment_entity::Entity::find()
        .filter(payment_entity::Column::Status.eq("pending"))
        .filter(payment_entity::Column::AttemptKey.is_not_null())
        .all(state.conn.as_ref()).await.map_err(|e| e.to_string())?;
    let mut count = 0;
    for payment in payments {
        let purchase = match purchase_entity::Entity::find_by_id(payment.purchase_id).one(state.conn.as_ref()).await {
            Ok(Some(p)) => p,
            _ => continue,
        };
        let outcome = if let Some(ref reference) = payment.gateway_reference {
            provider.status(reference).await.ok()
        } else if let Some(ref key) = payment.attempt_key {
            provider.search(&format!("torg-{}-{key}", purchase.id)).await.ok().flatten()
        } else { None };
        if let Some(outcome) = outcome {
            if apply_result(state, &purchase, &payment, outcome, provider.collector_id).await.is_ok() {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn spawn_payment_reconciliation(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = reconcile_unresolved(&state).await {
                log::debug!("Reconciliation: {e}");
            }
        }
    });
}

#[cfg(test)]
#[path = "../../tests/unit/endpoints/checkout_payment_tests.rs"]
mod tests;
