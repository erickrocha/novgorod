use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // C-018 (SR-TEN-006, SR-TEN-009): a marketplace cart has no tenant. The composite foreign
        // keys and the unique key of cart_item include tenant_id and are not enforced when it is
        // NULL, so the same guarantees are added on the single columns (cascade on cart delete,
        // existing SKU, one row per cart and SKU).
        manager
            .get_connection()
            .execute_unprepared(
                r#"
ALTER TABLE cart ALTER COLUMN tenant_id DROP NOT NULL;
ALTER TABLE cart_item ALTER COLUMN tenant_id DROP NOT NULL;
ALTER TABLE cart_item
    ADD CONSTRAINT fk_cart_item_cart_id FOREIGN KEY (cart_id) REFERENCES cart(id) ON DELETE CASCADE;
ALTER TABLE cart_item
    ADD CONSTRAINT fk_cart_item_sku_id FOREIGN KEY (sku_id) REFERENCES sku(id) ON DELETE RESTRICT;
CREATE UNIQUE INDEX uq_cart_item_cart_sku_any ON cart_item (cart_id, sku_id);
"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Tenant-less rows may exist, so the columns stay nullable on rollback.
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DROP INDEX IF EXISTS uq_cart_item_cart_sku_any;
ALTER TABLE cart_item DROP CONSTRAINT IF EXISTS fk_cart_item_sku_id;
ALTER TABLE cart_item DROP CONSTRAINT IF EXISTS fk_cart_item_cart_id;
"#,
            )
            .await?;
        Ok(())
    }
}
