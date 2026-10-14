use sea_orm_migration::prelude::*;

/// PagSeguro is the only payment provider (C-040). Tenant settings of the removed provider are
/// deleted: their credentials can no longer be used by any code path.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DELETE FROM tenant_payment_settings WHERE provider <> 'pagseguro';
ALTER TABLE tenant_payment_settings ALTER COLUMN provider SET DEFAULT 'pagseguro';
ALTER TABLE tenant_payment_settings
    ADD CONSTRAINT tenant_payment_settings_provider_check CHECK (provider = 'pagseguro');
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
ALTER TABLE tenant_payment_settings DROP CONSTRAINT tenant_payment_settings_provider_check;
ALTER TABLE tenant_payment_settings ALTER COLUMN provider SET DEFAULT 'mercado_pago';
"#,
            )
            .await?;
        Ok(())
    }
}
