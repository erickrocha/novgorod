use sea_orm_migration::prelude::*;

/// Buyer interest of the chosen installments (C-041): the charge is `amount_cents + interest_cents`;
/// `amount_cents` stays the purchase total that the seller allocations add up to.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE payment ADD COLUMN interest_cents BIGINT NOT NULL DEFAULT 0 CHECK (interest_cents >= 0);",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE payment DROP COLUMN interest_cents;")
            .await?;
        Ok(())
    }
}
