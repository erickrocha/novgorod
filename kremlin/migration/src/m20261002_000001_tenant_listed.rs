use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Listing flag (NOV-PLT-017): off for new and existing tenants, fail closed.
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE tenant ADD COLUMN IF NOT EXISTS listed BOOLEAN NOT NULL DEFAULT FALSE;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE tenant DROP COLUMN IF EXISTS listed;")
            .await?;
        Ok(())
    }
}
