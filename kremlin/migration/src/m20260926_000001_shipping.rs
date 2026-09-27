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
CREATE TABLE tenant_shipping_settings (
 tenant_id BIGINT PRIMARY KEY REFERENCES tenant(id),
 version BIGINT NOT NULL DEFAULT 1 CHECK(version > 0),
 credential_version BIGINT NOT NULL DEFAULT 0,
 configuration JSONB NOT NULL,
 credentials BYTEA,
 key_version VARCHAR(100),
 CHECK ((credentials IS NULL) = (key_version IS NULL))
);
ALTER TABLE orders ADD COLUMN shipping_snapshot JSONB;
"#,
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared("ALTER TABLE orders DROP COLUMN shipping_snapshot; DROP TABLE tenant_shipping_settings;").await?;
        Ok(())
    }
}
