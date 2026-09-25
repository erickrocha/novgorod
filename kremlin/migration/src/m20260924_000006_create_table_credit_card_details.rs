use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
 async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared(r#"
CREATE TABLE credit_card_details (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE,
    payment_id BIGINT NOT NULL UNIQUE REFERENCES payment(id),
    cardholder_name VARCHAR(255) NOT NULL,
    brand VARCHAR(50) NOT NULL,
    last_four_digits VARCHAR(4) NOT NULL CHECK (last_four_digits ~ '^[0-9]{4}$'),
    expiration_month INTEGER NOT NULL CHECK (expiration_month BETWEEN 1 AND 12),
    expiration_year INTEGER NOT NULL CHECK (expiration_year >= 2000),
    created_at TIMESTAMP NOT NULL,
    created_by VARCHAR(255),
    updated_at TIMESTAMP NOT NULL,
    updated_by VARCHAR(255)
);

"#).await?;
 Ok(())
 }
 async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared("DROP TABLE credit_card_details").await?;
 Ok(())
 }
}
