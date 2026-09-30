use super::checkout_quote_endpoint::error;
use crate::{
    AppState,
    commons::{exception_response::HttpResponse, i18n::Locale},
    infrastructure::payment_credentials::{
        MercadoPagoCredentials, PagSeguroCredentials, TenantCredentialsPayload,
    },
};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use business::{
    domain::{enums::Role, marketplace::PurchaseError, user::User},
    sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MercadoPagoCredentialsUpdate {
    #[schema(write_only)]
    pub access_token: Option<String>,
    pub public_key: Option<String>,
    pub collector_id: Option<i64>,
    pub webhook_secret: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PagSeguroCredentialsUpdate {
    #[schema(write_only)]
    pub token: Option<String>,
    pub public_key: Option<String>,
    pub environment: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaymentCredentialsUpdate {
    pub mercado_pago: Option<MercadoPagoCredentialsUpdate>,
    pub pagseguro: Option<PagSeguroCredentialsUpdate>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaymentSettingsUpdate {
    pub provider: String, // "mercado_pago" | "pagseguro"
    #[schema(write_only)]
    pub credentials: Option<PaymentCredentialsUpdate>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSettingsResponse {
    pub provider: String,
    pub version: i64,
    pub credentials_configured: bool,
    pub public_key: Option<String>,
    pub environment: Option<String>,
}

pub fn authorize(user: &User, tenant_id: i64) -> Result<(), PurchaseError> {
    if user.role == Role::SysAdmin
        || (user.role == Role::TenantOwner && user.tenant_id == Some(tenant_id))
    {
        Ok(())
    } else {
        Err(PurchaseError::Forbidden)
    }
}

#[utoipa::path(
    get,
    path = "/tenants/{tenantId}/payment-settings",
    params(("tenantId" = i64, Path)),
    responses(
        (status = 200, body = PaymentSettingsResponse),
        (status = 403, description = "Tenant owner or admin required"),
        (status = 404, description = "Seller not found")
    ),
    security(("bearer_auth" = [])),
    tag = "Payment"
)]
pub async fn get(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<PaymentSettingsResponse>> {
    async {
        authorize(&user, id)?;
        let row = state
            .conn
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT provider, version, credentials IS NOT NULL AS configured, configuration FROM tenant_payment_settings WHERE tenant_id=$1",
                [id.into()],
            ))
            .await?;

        if let Some(r) = row {
            let provider: String = r.try_get("", "provider")?;
            let version: i64 = r.try_get("", "version")?;
            let configured: bool = r.try_get("", "configured")?;
            let config_json: serde_json::Value = r.try_get("", "configuration")?;
            let public_key = config_json.get("publicKey").and_then(|v| v.as_str()).map(str::to_owned);
            let environment = config_json.get("environment").and_then(|v| v.as_str()).map(str::to_owned);

            Ok(Json(PaymentSettingsResponse {
                provider,
                version,
                credentials_configured: configured,
                public_key,
                environment,
            }))
        } else {
            Ok(Json(PaymentSettingsResponse {
                provider: "mercado_pago".into(),
                version: 0,
                credentials_configured: false,
                public_key: None,
                environment: None,
            }))
        }
    }
    .await
    .map_err(|e| error(locale, e))
}

#[utoipa::path(
    put,
    path = "/tenants/{tenantId}/payment-settings",
    params(("tenantId" = i64, Path)),
    request_body = PaymentSettingsUpdate,
    responses(
        (status = 204, description = "Settings saved; omitted credentials preserved"),
        (status = 400, description = "Invalid configuration"),
        (status = 403, description = "Tenant owner or admin required"),
        (status = 404, description = "Seller not found"),
        (status = 503, description = "Encryption keys unavailable")
    ),
    security(("bearer_auth" = [])),
    tag = "Payment"
)]
pub async fn put(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Path(id): Path<i64>,
    Json(input): Json<PaymentSettingsUpdate>,
) -> HttpResponse<StatusCode> {
    async {
        authorize(&user, id)?;
        let provider = input.provider.to_lowercase();
        if provider != "mercado_pago" && provider != "pagseguro" {
            return Err(PurchaseError::Validation("provedor de pagamento inválido"));
        }

        let tx = state.conn.begin().await?;
        let tenant = tx
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT id FROM tenant WHERE id=$1 FOR UPDATE",
                [id.into()],
            ))
            .await?
            .ok_or(PurchaseError::NotFound)?;
        let _ = tenant;

        let old = tx
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT provider, credentials, key_version, configuration FROM tenant_payment_settings WHERE tenant_id=$1",
                [id.into()],
            ))
            .await?;

        let mut bytes: Option<Vec<u8>> = old.as_ref().map(|r| r.try_get("", "credentials")).transpose()?.flatten();
        let mut key: Option<String> = old.as_ref().map(|r| r.try_get("", "key_version")).transpose()?.flatten();
        let old_config: serde_json::Value = old.as_ref().map(|r| r.try_get("", "configuration")).transpose()?.unwrap_or_else(|| serde_json::json!({}));
        let mut new_config = old_config;

        let changed = input.credentials.is_some();
        if let Some(cred_update) = input.credentials {
            if provider == "pagseguro" {
                let mut current = match (&bytes, &key) {
                    (Some(b), Some(k)) => match state.payment_keys.decrypt_payload(id, k, b) {
                        Ok(TenantCredentialsPayload::PagSeguro(ps)) => ps,
                        _ => PagSeguroCredentials::default(),
                    },
                    _ => PagSeguroCredentials::default(),
                };
                if let Some(ps_upd) = cred_update.pagseguro {
                    if let Some(token) = ps_upd.token { current.token = token; }
                    if let Some(pk) = ps_upd.public_key {
                        current.public_key = Some(pk.clone());
                        new_config["publicKey"] = serde_json::Value::String(pk);
                    }
                    if let Some(env) = ps_upd.environment {
                        current.environment = Some(env.clone());
                        new_config["environment"] = serde_json::Value::String(env);
                    }
                }
                current.validate()?;
                let payload = TenantCredentialsPayload::PagSeguro(current);
                let (active_key, encrypted_bytes) = state.payment_keys.encrypt_payload(id, &payload)?;
                key = Some(active_key);
                bytes = Some(encrypted_bytes);
            } else {
                let mut current = match (&bytes, &key) {
                    (Some(b), Some(k)) => match state.payment_keys.decrypt_payload(id, k, b) {
                        Ok(TenantCredentialsPayload::MercadoPago(mp)) => mp,
                        _ => MercadoPagoCredentials::default(),
                    },
                    _ => MercadoPagoCredentials::default(),
                };
                if let Some(mp_upd) = cred_update.mercado_pago {
                    if let Some(token) = mp_upd.access_token { current.access_token = token; }
                    if let Some(pk) = mp_upd.public_key {
                        current.public_key = Some(pk.clone());
                        new_config["publicKey"] = serde_json::Value::String(pk);
                    }
                    if let Some(cid) = mp_upd.collector_id {
                        current.collector_id = Some(cid);
                        new_config["collectorId"] = serde_json::json!(cid);
                    }
                    if let Some(ws) = mp_upd.webhook_secret { current.webhook_secret = Some(ws); }
                }
                current.validate()?;
                let payload = TenantCredentialsPayload::MercadoPago(current);
                let (active_key, encrypted_bytes) = state.payment_keys.encrypt_payload(id, &payload)?;
                key = Some(active_key);
                bytes = Some(encrypted_bytes);
            }
        }

        tx.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO tenant_payment_settings (tenant_id, provider, configuration, credentials, key_version, credential_version) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (tenant_id) DO UPDATE SET \
             provider = $2, configuration = $3, credentials = $4, key_version = $5, \
             version = tenant_payment_settings.version + 1, \
             credential_version = tenant_payment_settings.credential_version + $6",
            [
                id.into(),
                provider.into(),
                new_config.into(),
                bytes.into(),
                key.into(),
                i64::from(changed).into(),
            ],
        ))
        .await?;

        tx.commit().await?;
        Ok(StatusCode::NO_CONTENT)
    }
    .await
    .map_err(|e| error(locale, e))
}

#[cfg(test)]
#[path = "../../tests/unit/endpoints/payment_settings_endpoint_test.rs"]
mod tests;
