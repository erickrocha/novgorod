use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
 async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared(r#"
CREATE TABLE payment_transaction (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE,
    payment_id BIGINT NOT NULL REFERENCES payment(id),
    operation VARCHAR(30) NOT NULL,
    status VARCHAR(30) NOT NULL,
    amount_cents BIGINT NOT NULL CHECK (amount_cents >= 0),
    currency VARCHAR(3) NOT NULL CHECK (currency = 'BRL'),
    idempotency_key VARCHAR(128) NOT NULL,
    gateway_provider VARCHAR(100),
    gateway_transaction_id VARCHAR(255),
    authorization_code VARCHAR(100),
    response_code VARCHAR(100),
    response_message VARCHAR(500),
    created_at TIMESTAMP NOT NULL,
    created_by VARCHAR(255),
    UNIQUE (payment_id, operation, idempotency_key),
    UNIQUE (gateway_provider, gateway_transaction_id, operation)
);

"#).await?;
 Ok(())
 }
 async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared("DROP TABLE payment_transaction").await?;
 Ok(())
 }
}
