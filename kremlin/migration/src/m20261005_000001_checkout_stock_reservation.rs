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
CREATE TABLE checkout_stock_reservation (
    id BIGSERIAL PRIMARY KEY,
    purchase_id BIGINT NOT NULL REFERENCES purchase(id) ON DELETE CASCADE,
    order_id BIGINT NOT NULL,
    sku_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    status VARCHAR(20) NOT NULL CHECK (status IN ('reserved', 'consumed', 'released')),
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (purchase_id, sku_id),
    FOREIGN KEY (purchase_id, order_id) REFERENCES orders(purchase_id, id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id, order_id) REFERENCES orders(tenant_id, id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id, sku_id) REFERENCES sku_stock(tenant_id, sku_id) ON DELETE RESTRICT
);
CREATE INDEX idx_checkout_stock_reservation_expiry
    ON checkout_stock_reservation(status, expires_at);
"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE checkout_stock_reservation")
            .await?;
        Ok(())
    }
}