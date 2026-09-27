use crate::domain::{marketplace::PurchaseError, shipping::ShippingConfig};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
pub async fn configuration<C: ConnectionTrait>(
    db: &C,
    tenant_id: i64,
) -> Result<(i64, ShippingConfig), PurchaseError> {
    let row = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT version, configuration FROM tenant_shipping_settings WHERE tenant_id=$1",
            [tenant_id.into()],
        ))
        .await?;
    match row {
        None => Ok((0, ShippingConfig::default())),
        Some(row) => Ok((
            row.try_get("", "version")?,
            serde_json::from_value(row.try_get("", "configuration")?)
                .map_err(|_| PurchaseError::Validation("invalid shipping configuration"))?,
        )),
    }
}
