use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
CREATE TABLE purchase (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE,
    customer_id BIGINT NOT NULL REFERENCES customer(id),
    customer_name VARCHAR(500) NOT NULL,
    customer_tax_id VARCHAR(50) NOT NULL,
    customer_email VARCHAR(500),
    customer_phone VARCHAR(20),
    currency VARCHAR(3) NOT NULL CHECK (currency = 'BRL'),
    status VARCHAR(30) NOT NULL,
    subtotal_cents BIGINT NOT NULL CHECK (subtotal_cents >= 0),
    discount_cents BIGINT NOT NULL CHECK (discount_cents >= 0),
    shipping_cents BIGINT NOT NULL CHECK (shipping_cents >= 0),
    tax_total_cents BIGINT NOT NULL CHECK (tax_total_cents >= 0),
    total_cents BIGINT NOT NULL CHECK (total_cents >= 0),
    idempotency_key VARCHAR(128) NOT NULL,
    request_hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    created_by VARCHAR(255),
    updated_at TIMESTAMP NOT NULL,
    updated_by VARCHAR(255),
    UNIQUE (customer_id, idempotency_key),
    UNIQUE (id, customer_id),
    CHECK (total_cents = subtotal_cents - discount_cents + shipping_cents + tax_total_cents)
);
CREATE INDEX idx_purchase_customer_date ON purchase(customer_id, created_at, id);
"#,
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE purchase")
            .await?;
        Ok(())
    }
}
