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
CREATE TABLE IF NOT EXISTS warehouse (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL REFERENCES tenant(id) ON DELETE RESTRICT,
    name VARCHAR(100) NOT NULL,
    origin_cep VARCHAR(8) NOT NULL,
    street VARCHAR(255),
    number VARCHAR(50),
    complement VARCHAR(100),
    district VARCHAR(100),
    city VARCHAR(100) NOT NULL,
    uf VARCHAR(2) NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_by VARCHAR(255)
);

CREATE INDEX IF NOT EXISTS idx_warehouse_tenant_active ON warehouse(tenant_id, active);
CREATE INDEX IF NOT EXISTS idx_warehouse_tenant_default ON warehouse(tenant_id, is_default);

ALTER TABLE sku_stock
    ADD COLUMN IF NOT EXISTS warehouse_id BIGINT REFERENCES warehouse(id) ON DELETE RESTRICT;

ALTER TABLE shipping_rate
    DROP CONSTRAINT IF EXISTS uq_shipping_rate_tenant_uf,
    ADD COLUMN IF NOT EXISTS origin_warehouse_id BIGINT REFERENCES warehouse(id) ON DELETE RESTRICT,
    ADD COLUMN IF NOT EXISTS region_name VARCHAR(100),
    ADD COLUMN IF NOT EXISTS destination_cep_start VARCHAR(8),
    ADD COLUMN IF NOT EXISTS destination_cep_end VARCHAR(8),
    ADD COLUMN IF NOT EXISTS transit_days_min INTEGER NOT NULL DEFAULT 1 CHECK (transit_days_min >= 0),
    ADD COLUMN IF NOT EXISTS transit_days_max INTEGER NOT NULL DEFAULT 2 CHECK (transit_days_max >= transit_days_min),
    ADD COLUMN IF NOT EXISTS max_weight_g INTEGER CHECK (max_weight_g IS NULL OR max_weight_g > 0),
    ADD COLUMN IF NOT EXISTS extra_weight_per_kg_cents INTEGER CHECK (extra_weight_per_kg_cents IS NULL OR extra_weight_per_kg_cents >= 0),
    ADD COLUMN IF NOT EXISTS free_shipping_threshold_cents INTEGER CHECK (free_shipping_threshold_cents IS NULL OR free_shipping_threshold_cents >= 0);

CREATE INDEX IF NOT EXISTS idx_shipping_rate_destination_cep ON shipping_rate(tenant_id, destination_cep_start, destination_cep_end);
CREATE INDEX IF NOT EXISTS idx_shipping_rate_origin_warehouse ON shipping_rate(tenant_id, origin_warehouse_id);
"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DROP INDEX IF EXISTS idx_shipping_rate_origin_warehouse;
DROP INDEX IF EXISTS idx_shipping_rate_destination_cep;

ALTER TABLE shipping_rate
    DROP COLUMN IF EXISTS free_shipping_threshold_cents,
    DROP COLUMN IF EXISTS extra_weight_per_kg_cents,
    DROP COLUMN IF EXISTS max_weight_g,
    DROP COLUMN IF EXISTS transit_days_max,
    DROP COLUMN IF EXISTS transit_days_min,
    DROP COLUMN IF EXISTS destination_cep_end,
    DROP COLUMN IF EXISTS destination_cep_start,
    DROP COLUMN IF EXISTS region_name,
    DROP COLUMN IF EXISTS origin_warehouse_id,
    ADD CONSTRAINT uq_shipping_rate_tenant_uf UNIQUE (tenant_id, uf);

ALTER TABLE sku_stock
    DROP COLUMN IF EXISTS warehouse_id;

DROP TABLE IF EXISTS warehouse;
"#,
            )
            .await?;
        Ok(())
    }
}
