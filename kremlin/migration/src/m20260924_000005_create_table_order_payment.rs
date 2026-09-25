use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
 async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared(r#"
CREATE TABLE payment (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE,
    purchase_id BIGINT NOT NULL REFERENCES purchase(id),
    method VARCHAR(30) NOT NULL,
    status VARCHAR(30) NOT NULL,
    installments INTEGER NOT NULL CHECK (installments > 0),
    amount_cents BIGINT NOT NULL CHECK (amount_cents >= 0),
    currency VARCHAR(3) NOT NULL CHECK (currency = 'BRL'),
    gateway_provider VARCHAR(100),
    gateway_reference VARCHAR(255),
    created_at TIMESTAMP NOT NULL,
    created_by VARCHAR(255),
    updated_at TIMESTAMP NOT NULL,
    updated_by VARCHAR(255),
    UNIQUE (purchase_id, id),
    UNIQUE (gateway_provider, gateway_reference)
);
CREATE UNIQUE INDEX uq_payment_active_purchase ON payment(purchase_id) WHERE status IN ('pending_provider', 'pending', 'authorized', 'captured');
"#).await?;
 Ok(())
 }
 async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared("DROP TABLE payment").await?;
 Ok(())
 }
}
