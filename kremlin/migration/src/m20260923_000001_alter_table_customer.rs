use sea_orm_migration::{prelude::*, schema::*};
use crate::m20260917_000001_create_table_user::User;
use crate::m20260919_000002_create_table_customer::Customer;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop unique email constraint that depends on tenant_id
        manager
            .alter_table(
                Table::alter()
                    .table(Customer::Table)
                    .drop_constraint(Alias::new("uq_customer_tenant_email"))
                    .to_owned(),
            )
            .await?;

        // Alter table to make tenant_id nullable and add user_id
        manager
            .alter_table(
                Table::alter()
                    .table(Customer::Table)
                    .modify_column(big_integer(Customer::TenantId).null())
                    .add_column(big_integer(CustomerUserId::UserId).null())
                    .to_owned(),
            )
            .await?;

        // Add foreign key to user table
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_customer_user_id")
                    .from(Customer::Table, CustomerUserId::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_customer_user_id")
                    .table(Customer::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customer::Table)
                    .drop_column(CustomerUserId::UserId)
                    .modify_column(big_integer(Customer::TenantId).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_customer_tenant_email")
                    .table(Customer::Table)
                    .col(Customer::TenantId)
                    .col(Customer::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum CustomerUserId {
    UserId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drop_constraint_sql() {
        let stmt = Table::alter()
            .table(Customer::Table)
            .drop_constraint(Alias::new("uq_customer_tenant_email"))
            .to_string(PostgresQueryBuilder);
        println!("Generated SQL: {}", stmt);
        assert_eq!(
            stmt,
            r#"ALTER TABLE "customer" DROP CONSTRAINT "uq_customer_tenant_email""#
        );
    }
}
