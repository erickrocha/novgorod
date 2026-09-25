use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Also repair installations that applied the original cart migration.
        manager.get_connection().execute_unprepared(r#"
 ALTER TABLE cart DROP CONSTRAINT IF EXISTS fk_cart_customer;
 ALTER TABLE cart ADD CONSTRAINT fk_cart_customer FOREIGN KEY (customer_id) REFERENCES customer(id) ON DELETE SET NULL;
 "#).await?;
        Ok(())
    }
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Keep the corrected global-customer relationship on rollback. Restoring the
        // tenant/customer foreign key would invalidate existing global customer carts.
        Ok(())
    }
}
