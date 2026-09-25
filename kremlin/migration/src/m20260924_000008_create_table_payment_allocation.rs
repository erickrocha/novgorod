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
CREATE TABLE payment_allocation (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE,
    purchase_id BIGINT NOT NULL,
    payment_id BIGINT NOT NULL,
    order_id BIGINT NOT NULL,
    amount_cents BIGINT NOT NULL CHECK (amount_cents >= 0),
    created_at TIMESTAMP NOT NULL,
    created_by VARCHAR(255),
    updated_at TIMESTAMP NOT NULL,
    updated_by VARCHAR(255),
    FOREIGN KEY (purchase_id, payment_id) REFERENCES payment(purchase_id, id),
    FOREIGN KEY (purchase_id, order_id) REFERENCES orders(purchase_id, id),
    UNIQUE (payment_id, order_id)
);

"#,
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE payment_allocation")
            .await?;
        Ok(())
    }
}
