use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
 async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared(r#"
CREATE TABLE orders (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE,
    purchase_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL REFERENCES tenant(id),
    number VARCHAR(32) NOT NULL UNIQUE,
    customer_id BIGINT NOT NULL REFERENCES customer(id),
    customer_name VARCHAR(500) NOT NULL,
    customer_tax_id VARCHAR(50) NOT NULL,
    customer_email VARCHAR(500),
    customer_phone VARCHAR(20),
    status VARCHAR(30) NOT NULL,
    payment_status VARCHAR(30) NOT NULL,
    subtotal_cents BIGINT NOT NULL CHECK (subtotal_cents >= 0),
    discount_cents BIGINT NOT NULL CHECK (discount_cents >= 0),
    shipping_cents BIGINT NOT NULL CHECK (shipping_cents >= 0),
    tax_total_cents BIGINT NOT NULL CHECK (tax_total_cents >= 0),
    total_cents BIGINT NOT NULL CHECK (total_cents >= 0),
    coupon_id BIGINT,
    coupon_code VARCHAR(40),
    placed_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL,
    created_by VARCHAR(255),
    updated_at TIMESTAMP NOT NULL,
    updated_by VARCHAR(255),
    FOREIGN KEY (purchase_id, customer_id) REFERENCES purchase(id, customer_id),
    UNIQUE (purchase_id, tenant_id),
    UNIQUE (tenant_id, id),
    UNIQUE (purchase_id, id),
    FOREIGN KEY (tenant_id, coupon_id) REFERENCES coupon(tenant_id, id),
    CHECK (total_cents = subtotal_cents - discount_cents + shipping_cents + tax_total_cents)
);
CREATE INDEX idx_orders_customer_date ON orders(customer_id, placed_at, id);
CREATE INDEX idx_orders_tenant_status_date ON orders(tenant_id, status, placed_at, id);
"#).await?;
 Ok(())
 }
 async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared("DROP TABLE orders").await?;
 Ok(())
 }
}

#[derive(DeriveIden)]
pub enum Order {
 #[sea_orm(iden = "orders")]
 Table,
 Id,
 TenantId,
}
