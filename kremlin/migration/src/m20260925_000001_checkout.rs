use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(r#"
CREATE TABLE checkout_quote (
 id BIGSERIAL PRIMARY KEY,
 customer_id BIGINT NOT NULL REFERENCES customer(id),
 request JSONB NOT NULL,
 result JSONB NOT NULL,
 expires_at TIMESTAMP NOT NULL,
 purchase_id BIGINT UNIQUE REFERENCES purchase(id),
 created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_checkout_quote_customer ON checkout_quote(customer_id, expires_at);
CREATE TABLE checkout_coupon_reservation (
 id BIGSERIAL PRIMARY KEY,
 coupon_id BIGINT NOT NULL REFERENCES coupon(id),
 customer_id BIGINT NOT NULL REFERENCES customer(id),
 purchase_id BIGINT NOT NULL REFERENCES purchase(id),
 order_id BIGINT NOT NULL REFERENCES orders(id),
 status VARCHAR(20) NOT NULL CHECK (status IN ('reserved','redeemed','released')),
 expires_at TIMESTAMP NOT NULL,
 UNIQUE (coupon_id, purchase_id)
);
CREATE INDEX idx_checkout_coupon_active ON checkout_coupon_reservation(coupon_id, customer_id, status, expires_at);
ALTER TABLE payment ADD COLUMN attempt_key VARCHAR(128);
ALTER TABLE payment ADD COLUMN attempt_started_at TIMESTAMP;
ALTER TABLE payment ADD COLUMN attempt_error VARCHAR(100);
CREATE UNIQUE INDEX uq_payment_attempt_key ON payment(attempt_key) WHERE attempt_key IS NOT NULL;
ALTER TABLE credit_card_details ALTER COLUMN cardholder_name DROP NOT NULL;
ALTER TABLE credit_card_details ALTER COLUMN brand DROP NOT NULL;
ALTER TABLE credit_card_details ALTER COLUMN last_four_digits DROP NOT NULL;
ALTER TABLE credit_card_details ALTER COLUMN expiration_month DROP NOT NULL;
ALTER TABLE credit_card_details ALTER COLUMN expiration_year DROP NOT NULL;
"#).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(r#"
ALTER TABLE credit_card_details ALTER COLUMN expiration_year SET NOT NULL;
ALTER TABLE credit_card_details ALTER COLUMN expiration_month SET NOT NULL;
ALTER TABLE credit_card_details ALTER COLUMN last_four_digits SET NOT NULL;
ALTER TABLE credit_card_details ALTER COLUMN brand SET NOT NULL;
ALTER TABLE credit_card_details ALTER COLUMN cardholder_name SET NOT NULL;
DROP INDEX uq_payment_attempt_key;
ALTER TABLE payment DROP COLUMN attempt_error;
ALTER TABLE payment DROP COLUMN attempt_started_at;
ALTER TABLE payment DROP COLUMN attempt_key;
DROP TABLE checkout_coupon_reservation;
DROP TABLE checkout_quote;
"#).await?;
        Ok(())
    }
}
