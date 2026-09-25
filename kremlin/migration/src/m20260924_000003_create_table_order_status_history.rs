use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
 async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared(r#"
CREATE TABLE order_status_history (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    order_id BIGINT NOT NULL,
    from_status VARCHAR(30),
    to_status VARCHAR(30) NOT NULL,
    actor_type VARCHAR(20) NOT NULL,
    actor_id BIGINT,
    note TEXT,
    created_at TIMESTAMP NOT NULL,
    created_by VARCHAR(255),
    FOREIGN KEY (tenant_id, order_id) REFERENCES orders(tenant_id, id) ON DELETE CASCADE
);
CREATE INDEX idx_order_status_history_order ON order_status_history(order_id, id);
"#).await?;
 Ok(())
 }
 async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
 manager.get_connection().execute_unprepared("DROP TABLE order_status_history").await?;
 Ok(())
 }
}
