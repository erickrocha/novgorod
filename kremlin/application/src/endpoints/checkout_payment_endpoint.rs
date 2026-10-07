use crate::{
    AppState,
    infrastructure::{
        mercado_pago::MercadoPago,
        mock_payment::{MockPaymentProvider, MockProviderKind},
        pagseguro::PagSeguro,
        payment_credentials::TenantCredentialsPayload,
    },
};
use axum::{
    Json,
    body::Bytes,
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use business::{
    domain::{enums::Role, marketplace::PurchaseError, user::User},
    gateway::{
        cart_gateway::CartGateway,
        payment_provider_gateway::{
            ChargeRequest, PaymentProviderGateway, ProviderError, ProviderResult, ProviderStatus,
        },
        purchase_gateway::PurchaseGateway,
    },
    sea_orm::{
        ActiveModelTrait, ColumnTrait, ConnectionTrait, DbBackend, DbErr, EntityTrait, QueryFilter,
        Set, Statement, TransactionTrait,
    },
    use_cases::purchase_use_case::PurchaseUseCase,
};
use entity::{
    checkout_quote_entity, coupon_redemption_entity, credit_card_details_entity,
    order_status_history_entity, orders_entity, payment_entity, payment_transaction_entity,
    purchase_entity,
};
use serde::{Deserialize, Serialize};

type ApiError = (StatusCode, Json<serde_json::Value>);
fn bad(status: StatusCode, message: &str) -> ApiError {
    (status, Json(serde_json::json!({"message": message})))
}
fn unavailable() -> ApiError {
    bad(
        StatusCode::SERVICE_UNAVAILABLE,
        "Pagamento indisponível. Verifique a configuração do meio de pagamento.",
    )
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentConfig {
    pub provider: String,
    pub public_key: String,
}

pub async fn config(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PaymentConfig>, ApiError> {
    if user.role != Role::Customer {
        return Err(bad(StatusCode::FORBIDDEN, "Acesso negado"));
    }

    if let Some(tenant_id_str) = params.get("tenantId").or_else(|| params.get("tenant_id"))
        && let Ok(tid) = tenant_id_str.parse::<i64>()
            && let Ok(Some(row)) = state
                .conn
                .query_one_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "SELECT provider, configuration FROM tenant_payment_settings WHERE tenant_id=$1",
                    [tid.into()],
                ))
                .await
            {
                let provider: String = row.try_get("", "provider").unwrap_or_else(|_| "mercado_pago".into());
                let config_val: serde_json::Value = row.try_get("", "configuration").unwrap_or_default();
                if let Some(pk) = config_val.get("publicKey").and_then(|v| v.as_str())
                    && !pk.is_empty() {
                        return Ok(Json(PaymentConfig {
                            provider,
                            public_key: pk.to_owned(),
                        }));
                    }
            }

    if let Ok(Some(row)) = state
        .conn
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT provider, configuration FROM tenant_payment_settings ORDER BY version DESC LIMIT 1",
            [],
        ))
        .await
    {
        let provider: String = row.try_get("", "provider").unwrap_or_else(|_| "mercado_pago".into());
        let config_val: serde_json::Value = row.try_get("", "configuration").unwrap_or_default();
        if let Some(pk) = config_val.get("publicKey").and_then(|v| v.as_str())
            && !pk.is_empty() {
                return Ok(Json(PaymentConfig {
                    provider,
                    public_key: pk.to_owned(),
                }));
            }
    }

    if let Ok(public_key) = std::env::var("PAGSEGURO_PUBLIC_KEY")
        && !public_key.is_empty() {
            return Ok(Json(PaymentConfig {
                provider: "pagseguro".into(),
                public_key,
            }));
        }

    if let Ok(public_key) = std::env::var("MP_PUBLIC_KEY")
        && !public_key.is_empty() {
            return Ok(Json(PaymentConfig {
                provider: "mercado_pago".into(),
                public_key,
            }));
        }

    Err(unavailable())
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
pub struct PaymentState {
    purchase_id: i64,
    status: String,
    reference: Option<String>,
}

enum ResolvedProvider {
    MercadoPago(MercadoPago),
    PagSeguro(PagSeguro),
    Mock(MockPaymentProvider),
}

impl ResolvedProvider {
    fn name(&self) -> &'static str {
        match self {
            Self::MercadoPago(_) => "mercado_pago",
            Self::PagSeguro(_) => "pagseguro",
            Self::Mock(mock) => mock.name(),
        }
    }

    fn collector_id(&self) -> i64 {
        match self {
            Self::MercadoPago(mp) => mp.collector_id,
            Self::PagSeguro(_) => 0,
            Self::Mock(_) => 0,
        }
    }

    async fn charge(&self, request: ChargeRequest) -> Result<ProviderResult, ProviderError> {
        match self {
            Self::MercadoPago(mp) => mp.charge(request).await,
            Self::PagSeguro(ps) => ps.charge(request).await,
            Self::Mock(mock) => mock.charge(request).await,
        }
    }

    async fn status(&self, reference: &str) -> Result<ProviderResult, ProviderError> {
        match self {
            Self::MercadoPago(mp) => mp.status(reference).await,
            Self::PagSeguro(ps) => ps.status(reference).await,
            Self::Mock(mock) => mock.status(reference).await,
        }
    }

    async fn search(&self, external_reference: &str) -> Result<Option<ProviderResult>, ProviderError> {
        match self {
            Self::MercadoPago(mp) => mp.search(external_reference).await,
            Self::PagSeguro(ps) => ps.search(external_reference).await,
            Self::Mock(mock) => mock.search(external_reference).await,
        }
    }
}

fn mock_provider_for_config(
    mode: Option<&str>,
    provider: Option<&str>,
) -> Result<Option<MockPaymentProvider>, ApiError> {
    match mode {
        None | Some("") | Some("real") => Ok(None),
        Some("mock") => {
            let provider = provider.ok_or_else(unavailable)?;
            let kind = MockProviderKind::parse(provider).map_err(|_| unavailable())?;
            Ok(Some(MockPaymentProvider::new(kind)))
        }
        Some(_) => Err(unavailable()),
    }
}

fn configured_mock_provider() -> Result<Option<MockPaymentProvider>, ApiError> {
    match std::env::var("PAYMENT_PROVIDER_MODE") {
        Ok(mode) if mode == "mock" => {
            let provider = std::env::var("PAYMENT_MOCK_PROVIDER").map_err(|_| unavailable())?;
            mock_provider_for_config(Some(&mode), Some(&provider))
        }
        Ok(mode) => mock_provider_for_config(Some(&mode), None),
        Err(std::env::VarError::NotPresent) => mock_provider_for_config(None, None),
        Err(std::env::VarError::NotUnicode(_)) => Err(unavailable()),
    }
}

async fn resolve_provider(state: &AppState, purchase_id: i64) -> Result<ResolvedProvider, ApiError> {
    if let Some(provider) = configured_mock_provider()? {
        return Ok(ResolvedProvider::Mock(provider));
    }

    let order = orders_entity::Entity::find()
        .filter(orders_entity::Column::PurchaseId.eq(purchase_id))
        .one(state.conn.as_ref())
        .await
        .map_err(|_| unavailable())?;

    if let Some(order) = order {
        let tid = order.tenant_id;
        if let Ok(Some(row)) = state
            .conn
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT provider, credentials, key_version FROM tenant_payment_settings WHERE tenant_id=$1",
                [tid.into()],
            ))
            .await
        {
            let provider_name: String = row.try_get("", "provider").unwrap_or_else(|_| "mercado_pago".into());
            let credentials_bytes: Option<Vec<u8>> = row.try_get("", "credentials").ok().flatten();
            let key_version: Option<String> = row.try_get("", "key_version").ok().flatten();

            if let (Some(bytes), Some(key)) = (credentials_bytes, key_version)
                && let Ok(payload) = state.payment_keys.decrypt_payload(tid, &key, &bytes) {
                    match payload {
                        TenantCredentialsPayload::PagSeguro(ps) => {
                            if let Ok(ps_instance) =
                                PagSeguro::with_credentials(ps.token, ps.environment.as_deref())
                            {
                                return Ok(ResolvedProvider::PagSeguro(ps_instance));
                            }
                        }
                        TenantCredentialsPayload::MercadoPago(_mp) => {
                            if let Ok(mp_instance) = MercadoPago::configured() {
                                return Ok(ResolvedProvider::MercadoPago(mp_instance));
                            }
                        }
                    }
                }

            if provider_name == "pagseguro"
                && let Ok(ps) = PagSeguro::configured() {
                    return Ok(ResolvedProvider::PagSeguro(ps));
                }
        }
    }

    if let Ok(mp) = MercadoPago::configured() {
        return Ok(ResolvedProvider::MercadoPago(mp));
    }
    if let Ok(ps) = PagSeguro::configured() {
        return Ok(ResolvedProvider::PagSeguro(ps));
    }

    Err(unavailable())
}

async fn owned(
    state: &AppState,
    user: &User,
    id: i64,
) -> Result<(purchase_entity::Model, payment_entity::Model), ApiError> {
    if user.role != Role::Customer {
        return Err(bad(StatusCode::FORBIDDEN, "Acesso negado"));
    }
    let use_case = PurchaseUseCase::new(PurchaseGateway::new(state.conn.as_ref().clone()));
    use_case.get(user, id).await.map_err(|e| match e {
        PurchaseError::NotFound | PurchaseError::Forbidden => {
            bad(StatusCode::NOT_FOUND, "Compra não encontrada")
        }
        _ => bad(
            StatusCode::SERVICE_UNAVAILABLE,
            "Não foi possível consultar a compra",
        ),
    })?;
    let purchase = purchase_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(|| bad(StatusCode::NOT_FOUND, "Compra não encontrada"))?;
    let payment = payment_entity::Entity::find()
        .filter(payment_entity::Column::PurchaseId.eq(id))
        .one(state.conn.as_ref())
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(|| bad(StatusCode::NOT_FOUND, "Pagamento não encontrado"))?;
    Ok((purchase, payment))
}

fn state(purchase_id: i64, payment: &payment_entity::Model) -> PaymentState {
    PaymentState {
        purchase_id,
        status: payment.status.clone(),
        reference: payment.gateway_reference.clone(),
    }
}

fn validate_provider(
    result: &ProviderResult,
    purchase: &purchase_entity::Model,
    payment: &payment_entity::Model,
    collector_id: i64,
) -> Result<(), ApiError> {
    if result.external_reference
        != format!(
            "torg-{}-{}",
            purchase.id,
            payment.attempt_key.as_deref().unwrap_or_default()
        )
        || result.amount_cents != payment.amount_cents
        || result.currency != "BRL"
        || (collector_id != 0 && result.collector_id != collector_id)
        || result.payment_type_id != "credit_card"
    {
        return Err(bad(
            StatusCode::BAD_GATEWAY,
            "Resposta de pagamento inconsistente",
        ));
    }
    Ok(())
}

async fn apply_result(
    state: &AppState,
    purchase: &purchase_entity::Model,
    payment: &payment_entity::Model,
    result: ProviderResult,
    collector_id: i64,
    provider_name: &str,
) -> Result<PaymentState, ApiError> {
    validate_provider(&result, purchase, payment, collector_id)?;
    let tx = state.conn.begin().await.map_err(|_| unavailable())?;
    tx.query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT id FROM payment WHERE id=$1 FOR UPDATE",
        [payment.id.into()],
    ))
    .await
    .map_err(|_| unavailable())?;
    let current = payment_entity::Entity::find_by_id(payment.id)
        .one(&tx)
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(|| bad(StatusCode::NOT_FOUND, "Pagamento não encontrado"))?;
    if current.status == "captured"
        || (current.status == "failed" && result.status != ProviderStatus::Captured)
    {
        tx.commit().await.map_err(|_| unavailable())?;
        return Ok(self::state(purchase.id, &current));
    }
    if current
        .gateway_reference
        .as_deref()
        .is_some_and(|r| r != result.reference)
    {
        return Err(bad(StatusCode::CONFLICT, "Referência de pagamento divergente"));
    }
    let status = match result.status {
        ProviderStatus::Captured => "captured",
        ProviderStatus::Failed => "failed",
        ProviderStatus::Authorized => "authorized",
        ProviderStatus::Pending => "pending",
    };
    let mut update: payment_entity::ActiveModel = current.into();
    update.status = Set(status.into());
    update.gateway_provider = Set(Some(provider_name.into()));
    update.gateway_reference = Set(Some(result.reference.clone()));
    update.attempt_error = Set(None);
    let saved = update.update(&tx).await.map_err(|_| unavailable())?;
    tx.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment_transaction SET status=$1,gateway_transaction_id=$2 WHERE payment_id=$3 AND idempotency_key=$4",
        [
            status.into(),
            result.reference.clone().into(),
            saved.id.into(),
            saved.attempt_key.clone().unwrap_or_default().into(),
        ],
    ))
    .await
    .map_err(|_| unavailable())?;
    if let Some(ref card_meta) = result.card {
        let existing_card = credit_card_details_entity::Entity::find()
            .filter(credit_card_details_entity::Column::PaymentId.eq(saved.id))
            .one(&tx)
            .await
            .map_err(|_| unavailable())?;
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
            }
            .insert(&tx)
            .await
            .map_err(|_| unavailable())?;
        }
    }
    if matches!(
        result.status,
        ProviderStatus::Captured | ProviderStatus::Failed
    ) {
        let (purchase_status, order_status, reservation_status) =
            if result.status == ProviderStatus::Captured {
                ("paid", "paid", "redeemed")
            } else {
                ("payment_failed", "payment_failed", "released")
            };
        tx.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE purchase SET status=$1,updated_at=CURRENT_TIMESTAMP WHERE id=$2",
            [purchase_status.into(), purchase.id.into()],
        ))
        .await
        .map_err(|_| unavailable())?;
        tx.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE orders SET status=$1,payment_status=$2,updated_at=CURRENT_TIMESTAMP WHERE purchase_id=$3",
            [order_status.into(), status.into(), purchase.id.into()],
        ))
        .await
        .map_err(|_| unavailable())?;
        tx.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE checkout_coupon_reservation SET status=$1 WHERE purchase_id=$2 AND status != 'redeemed'",
            [reservation_status.into(), purchase.id.into()],
        ))
        .await
        .map_err(|_| unavailable())?;
        let stock_reservations = tx
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT id,tenant_id,sku_id,quantity FROM checkout_stock_reservation WHERE purchase_id=$1 AND status='reserved' FOR UPDATE",
                [purchase.id.into()],
            ))
            .await
            .map_err(|_| unavailable())?;
        for reservation in stock_reservations {
            let reservation_id: i64 = reservation
                .try_get("", "id")
                .map_err(|_| unavailable())?;
            let tenant_id: i64 = reservation
                .try_get("", "tenant_id")
                .map_err(|_| unavailable())?;
            let sku_id: i64 = reservation
                .try_get("", "sku_id")
                .map_err(|_| unavailable())?;
            let quantity: i32 = reservation
                .try_get("", "quantity")
                .map_err(|_| unavailable())?;
            let stock_update = if result.status == ProviderStatus::Captured {
                "UPDATE sku_stock SET quantity=quantity-$3,reserved=reserved-$3,updated_at=CURRENT_TIMESTAMP WHERE tenant_id=$1 AND sku_id=$2 AND quantity >= $3 AND reserved >= $3"
            } else {
                "UPDATE sku_stock SET reserved=reserved-$3,updated_at=CURRENT_TIMESTAMP WHERE tenant_id=$1 AND sku_id=$2 AND reserved >= $3"
            };
            let changed = tx
                .execute_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    stock_update,
                    [tenant_id.into(), sku_id.into(), quantity.into()],
                ))
                .await
                .map_err(|_| unavailable())?;
            if changed.rows_affected() != 1 {
                return Err(unavailable());
            }
            let changed = tx
                .execute_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "UPDATE checkout_stock_reservation SET status=$1 WHERE id=$2 AND status='reserved'",
                    [
                        (if result.status == ProviderStatus::Captured {
                            "consumed"
                        } else {
                            "released"
                        })
                        .into(),
                        reservation_id.into(),
                    ],
                ))
                .await
                .map_err(|_| unavailable())?;
            if changed.rows_affected() != 1 {
                return Err(unavailable());
            }
        }
        if result.status == ProviderStatus::Captured {
            let released_stock = tx
                .query_all_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "SELECT id,tenant_id,sku_id,quantity FROM checkout_stock_reservation WHERE purchase_id=$1 AND status='released' ORDER BY tenant_id,sku_id FOR UPDATE",
                    [purchase.id.into()],
                ))
                .await
                .map_err(|_| unavailable())?;
            for reservation in released_stock {
                let reservation_id: i64 = reservation
                    .try_get("", "id")
                    .map_err(|_| unavailable())?;
                let tenant_id: i64 = reservation
                    .try_get("", "tenant_id")
                    .map_err(|_| unavailable())?;
                let sku_id: i64 = reservation
                    .try_get("", "sku_id")
                    .map_err(|_| unavailable())?;
                let quantity: i32 = reservation
                    .try_get("", "quantity")
                    .map_err(|_| unavailable())?;
                let consumed = tx
                    .execute_raw(Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        "UPDATE sku_stock SET quantity=quantity-$3,updated_at=CURRENT_TIMESTAMP WHERE tenant_id=$1 AND sku_id=$2 AND quantity-reserved >= $3",
                        [tenant_id.into(), sku_id.into(), quantity.into()],
                    ))
                    .await
                    .map_err(|_| unavailable())?;
                if consumed.rows_affected() != 1 {
                    log::error!(
                        "late payment capture oversold, purchase_id={}, sku_id={}, quantity={}, manual refund",
                        purchase.id,
                        sku_id,
                        quantity
                    );
                    continue;
                }
                let updated = tx
                    .execute_raw(Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        "UPDATE checkout_stock_reservation SET status='consumed' WHERE id=$1 AND status='released'",
                        [reservation_id.into()],
                    ))
                    .await
                    .map_err(|_| unavailable())?;
                if updated.rows_affected() != 1 {
                    log::error!(
                        "late payment capture stock row not consumed, purchase_id={}, sku_id={}, quantity={}",
                        purchase.id,
                        sku_id,
                        quantity
                    );
                }
            }
        }
        let rows = orders_entity::Entity::find()
            .filter(orders_entity::Column::PurchaseId.eq(purchase.id))
            .all(&tx)
            .await
            .map_err(|_| unavailable())?;
        for order in &rows {
            order_status_history_entity::ActiveModel {
                tenant_id: Set(order.tenant_id),
                order_id: Set(order.id),
                from_status: Set(Some("pending_payment".into())),
                to_status: Set(order_status.into()),
                actor_type: Set("payment_gateway".into()),
                actor_id: Set(None),
                note: Set(Some(format!("{provider_name}:{}", result.reference))),
                ..Default::default()
            }
            .insert(&tx)
            .await
            .map_err(|_| unavailable())?;
            if result.status == ProviderStatus::Captured
                && let Some(coupon_id) = order.coupon_id {
                    let savepoint = match tx.begin().await {
                        Ok(savepoint) => savepoint,
                        Err(e) => {
                            log::warn!(
                                "coupon redemption savepoint unavailable, purchase_id={}, order_id={}: {e}",
                                purchase.id,
                                order.id
                            );
                            continue;
                        }
                    };
                    let redemption = coupon_redemption_entity::ActiveModel {
                        tenant_id: Set(Some(order.tenant_id)),
                        coupon_id: Set(coupon_id),
                        order_id: Set(order.id),
                        customer_id: Set(purchase.customer_id),
                        ..Default::default()
                    }
                    .insert(&savepoint)
                    .await;
                    match redemption {
                        Ok(_) => {
                            if let Err(e) = savepoint.commit().await {
                                log::warn!(
                                    "coupon redemption savepoint release failed, purchase_id={}, order_id={}: {e}",
                                    purchase.id,
                                    order.id
                                );
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "coupon redemption insert failed, purchase_id={}, order_id={}: {e}",
                                purchase.id,
                                order.id
                            );
                            if let Err(e) = savepoint.rollback().await {
                                log::error!(
                                    "coupon redemption savepoint rollback failed, purchase_id={}, order_id={}: {e}",
                                    purchase.id,
                                    order.id
                                );
                            }
                        }
                    }
                }
        }
        if result.status == ProviderStatus::Captured {
            // C-034: the approved payment consumes the customer's carts (best effort, never fails it)
            let tenants: Vec<i64> = rows.iter().map(|order| order.tenant_id).collect();
            CartGateway::purge_purchased(&tx, purchase.id, purchase.customer_id, &tenants).await;
        }
    }
    tx.commit().await.map_err(|_| unavailable())?;
    Ok(self::state(purchase.id, &saved))
}

pub async fn submit(
    State(app): State<AppState>,
    Extension(user): Extension<User>,
    Extension(context): Extension<crate::commons::tenant_context::TenantContext>,
    Path(id): Path<i64>,
    Json(card): Json<SubmitCard>,
) -> Result<Json<PaymentState>, ApiError> {
    if card.token.is_empty()
        || card.token.len() > 1024
        || card.payment_method_id.is_empty()
        || card.installments < 1
        || card.installments > 24
    {
        return Err(bad(StatusCode::BAD_REQUEST, "Dados do cartão inválidos"));
    }
    let (purchase, payment) = owned(&app, &user, id).await?;
    if purchase.total_cents == 0 {
        return Ok(Json(state(id, &payment)));
    }
    let attempt_is_in_flight_or_complete = matches!(
        payment.status.as_str(),
        "captured" | "authorized" | "pending"
    ) || (payment.status == "pending_provider" && payment.attempt_key.is_some());
    if purchase.created_at + chrono::Duration::minutes(15) <= chrono::Utc::now().naive_utc()
        && !attempt_is_in_flight_or_complete
    {
        return Err(bad(
            StatusCode::CONFLICT,
            "Prazo para pagamento expirado. Faça um novo orçamento.",
        ));
    }
    let Some(quote) = checkout_quote_entity::Entity::find()
        .filter(checkout_quote_entity::Column::PurchaseId.eq(id))
        .one(app.conn.as_ref())
        .await
        .map_err(|_| unavailable())?
    else {
        return Err(bad(StatusCode::CONFLICT, "Compra sem orçamento validado"));
    };
    // SR-TEN-007: with a Selector/Fixed context the paid items must belong to that tenant
    if let Some(tenant_id) = context.tenant_id()
        && !business::use_cases::checkout_quote_use_case::quote_items_belong_to(
            &quote.result,
            tenant_id,
        )
    {
        return Err(bad(StatusCode::BAD_REQUEST, "Item de outro tenant"));
    }

    let provider = resolve_provider(&app, id).await?;
    let provider_name = provider.name();

    let tx = app.conn.begin().await.map_err(|_| unavailable())?;
    tx.query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT id FROM payment WHERE id=$1 FOR UPDATE",
        [payment.id.into()],
    ))
    .await
    .map_err(|_| unavailable())?;
    let current = payment_entity::Entity::find_by_id(payment.id)
        .one(&tx)
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(|| bad(StatusCode::NOT_FOUND, "Pagamento não encontrado"))?;
    if current.status == "captured"
        || current.status == "authorized"
        || current.status == "pending"
        || (current.attempt_key.is_some() && current.status == "pending_provider")
    {
        tx.commit().await.map_err(|_| unavailable())?;
        return Ok(Json(state(id, &current)));
    }
    let released_stock = tx
        .query_all_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id,tenant_id,sku_id,quantity FROM checkout_stock_reservation WHERE purchase_id=$1 AND status='released' ORDER BY tenant_id,sku_id FOR UPDATE",
            [purchase.id.into()],
        ))
        .await
        .map_err(|_| unavailable())?;
    for reservation in released_stock {
        let reservation_id: i64 = reservation.try_get("", "id").map_err(|_| unavailable())?;
        let tenant_id: i64 = reservation
            .try_get("", "tenant_id")
            .map_err(|_| unavailable())?;
        let sku_id: i64 = reservation
            .try_get("", "sku_id")
            .map_err(|_| unavailable())?;
        let quantity: i32 = reservation
            .try_get("", "quantity")
            .map_err(|_| unavailable())?;
        let stock = tx
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT id FROM sku_stock WHERE tenant_id=$1 AND sku_id=$2 FOR UPDATE",
                [tenant_id.into(), sku_id.into()],
            ))
            .await
            .map_err(|_| unavailable())?;
        if stock.is_none() {
            return Err(bad(
                StatusCode::CONFLICT,
                "Estoque insuficiente para nova tentativa de pagamento",
            ));
        }
        let reserved = tx
            .execute_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "UPDATE sku_stock SET reserved=reserved+$3,updated_at=CURRENT_TIMESTAMP WHERE tenant_id=$1 AND sku_id=$2 AND quantity-reserved >= $3",
                [tenant_id.into(), sku_id.into(), quantity.into()],
            ))
            .await
            .map_err(|_| unavailable())?;
        if reserved.rows_affected() != 1 {
            return Err(bad(
                StatusCode::CONFLICT,
                "Estoque insuficiente para nova tentativa de pagamento",
            ));
        }
        let updated = tx
            .execute_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "UPDATE checkout_stock_reservation SET status='reserved',expires_at=CURRENT_TIMESTAMP+interval '15 minutes' WHERE id=$1 AND status='released'",
                [reservation_id.into()],
            ))
            .await
            .map_err(|_| unavailable())?;
        if updated.rows_affected() != 1 {
            return Err(unavailable());
        }
    }
    tx.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE checkout_coupon_reservation SET status='reserved' WHERE purchase_id=$1 AND status='released' AND expires_at > CURRENT_TIMESTAMP",
        [purchase.id.into()],
    ))
    .await
    .map_err(|_| unavailable())?;
    let key = uuid::Uuid::new_v4().to_string();
    let mut update: payment_entity::ActiveModel = current.into();
    update.attempt_key = Set(Some(key.clone()));
    update.attempt_started_at = Set(Some(chrono::Utc::now().naive_utc()));
    update.attempt_error = Set(None);
    update.status = Set("pending".into());
    update.installments = Set(card.installments);
    update.gateway_provider = Set(Some(provider_name.into()));
    update.gateway_reference = Set(None);
    let saved = update.update(&tx).await.map_err(|_| unavailable())?;
    payment_transaction_entity::ActiveModel {
        payment_id: Set(saved.id),
        operation: Set("charge".into()),
        status: Set("pending".into()),
        amount_cents: Set(saved.amount_cents),
        currency: Set("BRL".into()),
        idempotency_key: Set(key.clone()),
        gateway_provider: Set(Some(provider_name.into())),
        ..Default::default()
    }
    .insert(&tx)
    .await
    .map_err(|_| unavailable())?;
    tx.commit().await.map_err(|_| unavailable())?;
    let charge = ChargeRequest {
        payment_id: saved.id,
        amount_cents: saved.amount_cents,
        currency: "BRL".into(),
        idempotency_key: key.clone(),
        provider_token: card.token,
        external_reference: format!("torg-{id}-{key}"),
        payer_email: purchase.customer_email.clone().unwrap_or_default(),
        payer_tax_id: purchase.customer_tax_id.clone(),
        payment_method_id: card.payment_method_id,
        issuer_id: card.issuer_id,
        installments: card.installments,
    };
    match provider.charge(charge).await {
        Ok(result) => Ok(Json(
            apply_result(
                &app,
                &purchase,
                &saved,
                result,
                provider.collector_id(),
                provider_name,
            )
            .await?,
        )),
        Err(e) => {
            log::warn!(
                "{provider_name} payment attempt requires reconciliation: {}",
                e.0
            );
            Ok(Json(state(id, &saved)))
        }
    }
}

pub async fn status(
    State(app): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
) -> Result<Json<PaymentState>, ApiError> {
    let (purchase, payment) = owned(&app, &user, id).await?;
    if payment.status == "captured"
        || payment.status == "failed"
        || payment.status == "pending_provider"
    {
        return Ok(Json(state(id, &payment)));
    }
    let provider = resolve_provider(&app, id).await?;
    let outcome = if let Some(reference) = &payment.gateway_reference {
        Some(provider.status(reference).await.map_err(|_| unavailable())?)
    } else {
        provider
            .search(&format!(
                "torg-{id}-{}",
                payment.attempt_key.as_deref().unwrap_or_default()
            ))
            .await
            .map_err(|_| unavailable())?
    };
    if let Some(outcome) = outcome {
        Ok(Json(
            apply_result(
                &app,
                &purchase,
                &payment,
                outcome,
                provider.collector_id(),
                provider.name(),
            )
            .await?,
        ))
    } else {
        Ok(Json(state(id, &payment)))
    }
}

#[derive(Deserialize, Default)]
pub struct NotificationQuery {
    #[serde(rename = "data.id")]
    pub data_id: Option<String>,
    pub id: Option<String>,
}

pub fn verify_signature(
    secret: &str,
    signature: &str,
    request_id: &str,
    data_id: &str,
) -> Result<(), &'static str> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut ts = None;
    let mut v1 = None;
    for part in signature.split(',') {
        if let Some((k, v)) = part.trim().split_once('=') {
            match k {
                "ts" => ts = Some(v),
                "v1" => v1 = Some(v),
                _ => {}
            }
        }
    }
    let ts = ts.ok_or("Assinatura inválida")?;
    let expected = v1.ok_or("Assinatura inválida")?;
    let bytes = (0..expected.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(expected.get(i..i + 2).unwrap_or(""), 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Assinatura inválida")?;
    let manifest = format!("id:{data_id};request-id:{request_id};ts:{ts};");
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| "Assinatura inválida")?;
    mac.update(manifest.as_bytes());
    mac.verify_slice(&bytes).map_err(|_| "Assinatura inválida")?;
    Ok(())
}

pub async fn webhook(
    State(app): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NotificationQuery>,
    body_bytes: Bytes,
) -> Result<StatusCode, ApiError> {
    let secret = std::env::var("MP_WEBHOOK_SECRET").map_err(|_| unavailable())?;
    let signature = headers
        .get("x-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| bad(StatusCode::UNAUTHORIZED, "Assinatura ausente"))?;
    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| bad(StatusCode::UNAUTHORIZED, "Identificador ausente"))?;
    let mut data_id = query.data_id.or(query.id);
    if data_id.is_none() && !body_bytes.is_empty()
        && let Ok(json_body) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
            data_id = json_body
                .get("data")
                .and_then(|d| d.get("id"))
                .and_then(|v| {
                    v.as_str()
                        .or_else(|| v.as_i64().map(|n| Box::leak(n.to_string().into_boxed_str()) as &str))
                })
                .map(str::to_owned)
                .or_else(|| {
                    json_body.get("id").and_then(|v| {
                        v.as_str()
                            .or_else(|| v.as_i64().map(|n| Box::leak(n.to_string().into_boxed_str()) as &str))
                    }).map(str::to_owned)
                });
        }
    let data_id = data_id.ok_or_else(|| bad(StatusCode::UNAUTHORIZED, "Pagamento ausente"))?;
    if !data_id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(bad(StatusCode::UNAUTHORIZED, "Pagamento inválido"));
    }
    verify_signature(&secret, signature, request_id, &data_id)
        .map_err(|msg| bad(StatusCode::UNAUTHORIZED, msg))?;
    let provider = MercadoPago::configured().map_err(|_| unavailable())?;
    let outcome = provider.status(&data_id).await.map_err(|_| unavailable())?;
    let id = outcome
        .external_reference
        .strip_prefix("torg-")
        .and_then(|s| s.split_once('-'))
        .and_then(|(id, _)| id.parse::<i64>().ok());
    let Some(id) = id else { return Ok(StatusCode::OK); };
    let purchase = purchase_entity::Entity::find_by_id(id)
        .one(app.conn.as_ref())
        .await
        .map_err(|_| unavailable())?;
    let Some(purchase) = purchase else { return Ok(StatusCode::OK); };
    let payment = payment_entity::Entity::find()
        .filter(payment_entity::Column::PurchaseId.eq(id))
        .one(app.conn.as_ref())
        .await
        .map_err(|_| unavailable())?;
    let Some(payment) = payment else { return Ok(StatusCode::OK); };
    apply_result(
        &app,
        &purchase,
        &payment,
        outcome,
        provider.collector_id,
        "mercado_pago",
    )
    .await?;
    Ok(StatusCode::OK)
}

pub async fn pagseguro_webhook(
    State(app): State<AppState>,
    body_bytes: Bytes,
) -> Result<StatusCode, ApiError> {
    if body_bytes.is_empty() {
        return Ok(StatusCode::OK);
    }
    let Ok(json_body) = serde_json::from_slice::<serde_json::Value>(&body_bytes) else {
        return Ok(StatusCode::OK);
    };

    let outcome = match PagSeguro::parse_payment_json(json_body) {
        Ok(res) => res,
        Err(_) => return Ok(StatusCode::OK),
    };

    let id = outcome
        .external_reference
        .strip_prefix("torg-")
        .and_then(|s| s.split_once('-'))
        .and_then(|(id, _)| id.parse::<i64>().ok());
    let Some(id) = id else { return Ok(StatusCode::OK); };
    let purchase = purchase_entity::Entity::find_by_id(id)
        .one(app.conn.as_ref())
        .await
        .map_err(|_| unavailable())?;
    let Some(purchase) = purchase else { return Ok(StatusCode::OK); };
    let payment = payment_entity::Entity::find()
        .filter(payment_entity::Column::PurchaseId.eq(id))
        .one(app.conn.as_ref())
        .await
        .map_err(|_| unavailable())?;
    let Some(payment) = payment else { return Ok(StatusCode::OK); };

    apply_result(&app, &purchase, &payment, outcome, 0, "pagseguro").await?;
    Ok(StatusCode::OK)
}

pub async fn reconcile_unresolved(state: &AppState) -> Result<usize, String> {
    let payments = payment_entity::Entity::find()
        .filter(payment_entity::Column::Status.eq("pending"))
        .filter(payment_entity::Column::AttemptKey.is_not_null())
        .all(state.conn.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    let mut count = 0;
    for payment in payments {
        let purchase = match purchase_entity::Entity::find_by_id(payment.purchase_id)
            .one(state.conn.as_ref())
            .await
        {
            Ok(Some(p)) => p,
            _ => continue,
        };

        let provider = match resolve_provider(state, purchase.id).await {
            Ok(p) => p,
            Err(_) => continue,
        };

        let outcome = if let Some(ref reference) = payment.gateway_reference {
            provider.status(reference).await.ok()
        } else if let Some(ref key) = payment.attempt_key {
            provider
                .search(&format!("torg-{}-{key}", purchase.id))
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        if let Some(outcome) = outcome
            && apply_result(
                state,
                &purchase,
                &payment,
                outcome,
                provider.collector_id(),
                provider.name(),
            )
            .await
            .is_ok()
            {
                count += 1;
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

async fn release_expired_stock_reservations(state: &AppState) -> Result<u64, DbErr> {
    let tx = state.conn.begin().await?;
    let payments = tx
        .query_all_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT p.id,p.purchase_id FROM payment p WHERE ((p.status='pending_provider' AND p.attempt_key IS NULL) OR p.status='failed') AND EXISTS (SELECT 1 FROM checkout_stock_reservation r WHERE r.purchase_id=p.purchase_id AND r.status='reserved' AND r.expires_at <= CURRENT_TIMESTAMP) ORDER BY p.id FOR UPDATE OF p SKIP LOCKED".to_string(),
        ))
        .await?;
    let mut released_count = 0;
    for payment in payments {
        let purchase_id: i64 = payment.try_get("", "purchase_id")?;
        let reservations = tx
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT id,tenant_id,sku_id,quantity FROM checkout_stock_reservation WHERE purchase_id=$1 AND status='reserved' AND expires_at <= CURRENT_TIMESTAMP ORDER BY id FOR UPDATE",
                [purchase_id.into()],
            ))
            .await?;
        for reservation in reservations {
            let reservation_id: i64 = reservation.try_get("", "id")?;
            let tenant_id: i64 = reservation.try_get("", "tenant_id")?;
            let sku_id: i64 = reservation.try_get("", "sku_id")?;
            let quantity: i32 = reservation.try_get("", "quantity")?;
            let stock = tx
                .execute_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "UPDATE sku_stock SET reserved=reserved-$3,updated_at=CURRENT_TIMESTAMP WHERE tenant_id=$1 AND sku_id=$2 AND reserved >= $3",
                    [tenant_id.into(), sku_id.into(), quantity.into()],
                ))
                .await?;
            if stock.rows_affected() != 1 {
                return Err(DbErr::Custom(format!(
                    "stock reservation release mismatch, purchase_id={purchase_id}, sku_id={sku_id}"
                )));
            }
            let reservation_update = tx
                .execute_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "UPDATE checkout_stock_reservation SET status='released' WHERE id=$1 AND status='reserved'",
                    [reservation_id.into()],
                ))
                .await?;
            if reservation_update.rows_affected() != 1 {
                return Err(DbErr::Custom(format!(
                    "stock reservation status mismatch, purchase_id={purchase_id}, sku_id={sku_id}"
                )));
            }
            released_count += 1;
        }
        tx.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE checkout_coupon_reservation SET status='released' WHERE purchase_id=$1 AND status='reserved' AND expires_at <= CURRENT_TIMESTAMP",
            [purchase_id.into()],
        ))
        .await?;
    }
    tx.commit().await?;
    Ok(released_count)
}

pub fn spawn_stock_expiry(state: AppState) {
    let enabled = match std::env::var("STOCK_EXPIRY_JOB_ENABLED") {
        Ok(value) => match value.trim().parse::<bool>() {
            Ok(enabled) => enabled,
            Err(_) => {
                log::warn!("invalid STOCK_EXPIRY_JOB_ENABLED value; defaulting to enabled");
                true
            }
        },
        Err(_) => true,
    };
    if !enabled {
        log::info!("Stock expiry job disabled by STOCK_EXPIRY_JOB_ENABLED");
        return;
    }
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            match release_expired_stock_reservations(&state).await {
                Ok(count) if count > 0 => log::info!("Expired stock reservations released: {count}"),
                Ok(_) => {}
                Err(e) => log::warn!("Stock expiry job failed: {e}"),
            }
        }
    });
}

#[cfg(test)]
#[path = "../../tests/unit/endpoints/checkout_payment_tests.rs"]
mod tests;
