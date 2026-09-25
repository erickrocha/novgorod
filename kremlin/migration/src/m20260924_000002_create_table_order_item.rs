use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
 async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared(r#"
CREATE TABLE order_item (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    order_id BIGINT NOT NULL,
    sku_id BIGINT NOT NULL,
    sku_code VARCHAR(64) NOT NULL,
    product_name VARCHAR(200) NOT NULL,
    attributes_desc TEXT,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price_cents BIGINT NOT NULL CHECK (unit_price_cents >= 0),
    discount_cents BIGINT NOT NULL CHECK (discount_cents >= 0),
    ncm VARCHAR(8) NOT NULL,
    cfop VARCHAR(4),
    csosn VARCHAR(4),
    icms_rate_bp INTEGER NOT NULL CHECK (icms_rate_bp >= 0),
    tax_cents BIGINT NOT NULL CHECK (tax_cents >= 0),
    total_cents BIGINT NOT NULL CHECK (total_cents >= 0),
    created_at TIMESTAMP NOT NULL,
    created_by VARCHAR(255),
    updated_at TIMESTAMP NOT NULL,
    updated_by VARCHAR(255),
    FOREIGN KEY (tenant_id, order_id) REFERENCES orders(tenant_id, id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id, sku_id) REFERENCES sku(tenant_id, id),
    UNIQUE (order_id, sku_id),
    CHECK (total_cents = quantity::bigint * unit_price_cents - discount_cents + tax_cents)
);

"#).await?;
 Ok(())
 }
 async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared("DROP TABLE order_item").await?;
 Ok(())
 }
}
