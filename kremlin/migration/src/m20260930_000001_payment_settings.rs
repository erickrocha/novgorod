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
CREATE TABLE tenant_payment_settings (
    tenant_id BIGINT PRIMARY KEY REFERENCES tenant(id),
    provider VARCHAR(50) NOT NULL DEFAULT 'mercado_pago',
    version BIGINT NOT NULL DEFAULT 1 CHECK(version > 0),
    credential_version BIGINT NOT NULL DEFAULT 0,
    configuration JSONB NOT NULL DEFAULT '{}'::jsonb,
    credentials BYTEA,
    key_version VARCHAR(100),
    CHECK ((credentials IS NULL) = (key_version IS NULL))
);
"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE tenant_payment_settings;")
            .await?;
        Ok(())
    }
}
