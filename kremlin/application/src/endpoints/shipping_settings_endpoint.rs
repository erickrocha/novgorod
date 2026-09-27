use super::checkout_quote_endpoint::error;
use crate::{
    AppState,
    commons::{exception_response::HttpResponse, i18n::Locale},
    infrastructure::shipping_credentials::CorreiosCredentials,
};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use business::{
    domain::{
        enums::Role,
        marketplace::PurchaseError,
        shipping::{ShippingConfig, ShippingMode},
        user::User,
    },
    gateway::shipping_settings_gateway,
    sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CredentialsUpdate {
    #[schema(write_only)]
    pub username: Option<String>,
    #[schema(write_only)]
    pub api_access_code: Option<String>,
    #[schema(write_only)]
    pub posting_card: Option<String>,
    #[schema(write_only)]
    pub contract: Option<String>,
    #[schema(write_only)]
    pub regional_identifier: Option<String>,
}
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShippingSettingsUpdate {
    pub configuration: ShippingConfig,
    #[schema(write_only)]
    pub credentials: Option<CredentialsUpdate>,
}
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShippingSettingsResponse {
    pub configuration: ShippingConfig,
    pub version: i64,
    pub credentials_configured: bool,
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
#[utoipa::path(get, path="/tenants/{tenantId}/shipping-settings", params(("tenantId" = i64, Path)), responses((status=200, body=ShippingSettingsResponse), (status=403, description="Tenant owner or admin required"), (status=404, description="Seller not found")), security(("bearer_auth"=[])), tag="Shipping")]
pub async fn get(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<ShippingSettingsResponse>> {
    async {
        authorize(&user,id)?;
        let tenant = state.conn.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT postal_code FROM tenant WHERE id=$1",[id.into()])).await?.ok_or(PurchaseError::NotFound)?;
        let (version, mut configuration) = shipping_settings_gateway::configuration(state.conn.as_ref(), id).await?;
        if version == 0 { configuration.origin_cep = tenant.try_get::<Option<String>>("", "postal_code")?.and_then(|v| business::domain::shipping::normalize_cep(&v).ok()); }
        let row = state.conn.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres, "SELECT credentials IS NOT NULL AS configured FROM tenant_shipping_settings WHERE tenant_id=$1",[id.into()])).await?;
        let configured = row.map(|r| r.try_get::<bool>("", "configured")).transpose()?.unwrap_or(false);
        Ok(Json(ShippingSettingsResponse { configuration, version, credentials_configured: configured }))
    }.await.map_err(|e| error(locale,e))
}
#[utoipa::path(put, path="/tenants/{tenantId}/shipping-settings", params(("tenantId" = i64, Path)), request_body=ShippingSettingsUpdate, responses((status=204, description="Settings saved; omitted credentials preserved"), (status=400, description="Invalid configuration"), (status=403, description="Tenant owner or admin required"), (status=404, description="Seller not found"), (status=503, description="Encryption keys unavailable")), security(("bearer_auth"=[])), tag="Shipping")]
pub async fn put(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Path(id): Path<i64>,
    Json(mut input): Json<ShippingSettingsUpdate>,
) -> HttpResponse<StatusCode> {
    async {
        authorize(&user,id)?;
        let tx = state.conn.begin().await?;
        let tenant = tx.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT postal_code FROM tenant WHERE id=$1 FOR UPDATE",[id.into()])).await?.ok_or(PurchaseError::NotFound)?;
        if input.configuration.origin_cep.is_none() {
            input.configuration.origin_cep = tenant.try_get::<Option<String>>("", "postal_code")?.and_then(|v| business::domain::shipping::normalize_cep(&v).ok());
        }
        input.configuration.validate()?;
        let old = tx.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT credentials,key_version FROM tenant_shipping_settings WHERE tenant_id=$1",[id.into()])).await?;
        let mut bytes: Option<Vec<u8>> = old.as_ref().map(|r| r.try_get("", "credentials")).transpose()?.flatten();
        let mut key: Option<String> = old.as_ref().map(|r| r.try_get("", "key_version")).transpose()?.flatten();
        let changed = input.credentials.is_some();
        if let Some(update) = input.credentials {
            let mut credentials = match (&bytes, &key) {
                (Some(bytes), Some(key)) => state.shipping_keys.decrypt(id,key,bytes)?,
                _ => CorreiosCredentials::default(),
            };
            if let Some(v) = update.username { credentials.username = v; }
            if let Some(v) = update.api_access_code { credentials.api_access_code = v; }
            if let Some(v) = update.posting_card { credentials.posting_card = v; }
            if let Some(v) = update.contract { credentials.contract = v; }
            if let Some(v) = update.regional_identifier { credentials.regional_identifier = v; }
            credentials.validate()?;
            let encrypted = state.shipping_keys.encrypt(id, &credentials)?;
            key = Some(encrypted.0); bytes = Some(encrypted.1);
        }
        if input.configuration.mode == ShippingMode::Correios && bytes.is_none() { return Err(PurchaseError::Validation("Correios credentials required")); }
        let config = serde_json::to_value(input.configuration).map_err(|_| PurchaseError::Validation("invalid shipping settings"))?;
        tx.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,
            "INSERT INTO tenant_shipping_settings (tenant_id, configuration, credentials, key_version, credential_version) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (tenant_id) DO UPDATE SET configuration=$2,credentials=$3,key_version=$4,version=tenant_shipping_settings.version+1,credential_version=tenant_shipping_settings.credential_version+$5",
            [id.into(),config.into(),bytes.into(),key.into(),i64::from(changed).into()])).await?;
        tx.commit().await?;
        Ok(StatusCode::NO_CONTENT)
    }.await.map_err(|e| error(locale,e))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn user(role: Role, tenant_id: Option<i64>) -> User {
        User {
            id: Some(1),
            uuid: None,
            name: None,
            email: "test@example.com".into(),
            password: String::new(),
            enabled: true,
            first_login: false,
            role,
            tenant_id,
            created_at: None,
            updated_at: None,
            created_by: None,
            updated_by: None,
        }
    }
    #[test]
    fn only_matching_owner_and_sysadmin_can_manage_settings() {
        assert!(authorize(&user(Role::SysAdmin, None), 7).is_ok());
        assert!(authorize(&user(Role::TenantOwner, Some(7)), 7).is_ok());
        assert!(authorize(&user(Role::TenantOwner, Some(8)), 7).is_err());
        assert!(authorize(&user(Role::TenantUser, Some(7)), 7).is_err());
        assert!(authorize(&user(Role::Customer, Some(7)), 7).is_err());
        assert!(authorize(&user(Role::TenantOwner, None), 7).is_err());
    }
    #[test]
    fn settings_response_exposes_status_only() {
        let json = serde_json::to_value(ShippingSettingsResponse {
            configuration: ShippingConfig::default(),
            version: 1,
            credentials_configured: true,
        })
        .unwrap();
        assert_eq!(json["credentialsConfigured"], true);
        assert!(json.get("credentials").is_none());
        assert!(json.get("keyVersion").is_none());
    }
}
