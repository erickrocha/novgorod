use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SR-MKT-005: a purchase made in a tenant-bound context records that tenant. Marketplace
        // purchases stay NULL. Historical purchases whose orders all belong to one tenant are
        // backfilled; multi-seller history stays NULL.
        manager
            .get_connection()
            .execute_unprepared(
                r#"
ALTER TABLE purchase ADD COLUMN tenant_id BIGINT NULL REFERENCES tenant(id);
UPDATE purchase p SET tenant_id = o.tenant_id
FROM (
    SELECT purchase_id, MIN(tenant_id) AS tenant_id, COUNT(DISTINCT tenant_id) AS n
    FROM orders GROUP BY purchase_id
) o
WHERE o.purchase_id = p.id AND o.n = 1;
"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE purchase DROP COLUMN IF EXISTS tenant_id;")
            .await?;
        Ok(())
    }
}
