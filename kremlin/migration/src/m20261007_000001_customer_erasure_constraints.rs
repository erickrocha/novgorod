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
                ALTER TABLE purchase
                    DROP CONSTRAINT IF EXISTS purchase_customer_id_fkey;
                ALTER TABLE orders
                    DROP CONSTRAINT IF EXISTS orders_customer_id_fkey;
                ALTER TABLE customer
                    ADD CONSTRAINT fk_customer_user
                    FOREIGN KEY (user_id) REFERENCES "user"(id) ON DELETE RESTRICT;
                CREATE UNIQUE INDEX uq_customer_user_id_not_null
                    ON customer (user_id) WHERE user_id IS NOT NULL;
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
                ALTER TABLE purchase
                    ADD CONSTRAINT purchase_customer_id_fkey
                    FOREIGN KEY (customer_id) REFERENCES customer(id);
                ALTER TABLE orders
                    ADD CONSTRAINT orders_customer_id_fkey
                    FOREIGN KEY (customer_id) REFERENCES customer(id);
                DROP INDEX IF EXISTS uq_customer_user_id_not_null;
                ALTER TABLE customer
                    DROP CONSTRAINT IF EXISTS fk_customer_user;
                "#,
            )
            .await?;
        Ok(())
    }
}
