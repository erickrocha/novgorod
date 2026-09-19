use crate::m20260917_000002_create_tenant_table::Tenant;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Customer::Table)
                    .if_not_exists()
                    .col(pk_auto(Customer::Id).big_integer())
                    .col(uuid_uniq(Customer::Uuid))
                    .col(big_integer(Customer::TenantId))
                    .col(string_len(Customer::Name, 150))
                    .col(string_len(Customer::Email, 190))
                    .col(string_len(Customer::PasswordHash, 255))
                    .col(string_len_null(Customer::Cpf, 255))
                    .col(string_len_null(Customer::Phone, 20))
                    .col(boolean(Customer::MarketingConsent).default(false))
                    .col(date_time_null(Customer::ConsentAt))
                    .col(boolean(Customer::Active).default(true))
                    .col(date_time(Customer::CreatedAt))
                    .col(string_len_null(Customer::CreatedBy, 255))
                    .col(date_time(Customer::UpdatedAt))
                    .col(string_len_null(Customer::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_customer_tenant_id_id")
                            .col(Customer::TenantId)
                            .col(Customer::Id)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_customer_tenant_email")
                            .col(Customer::TenantId)
                            .col(Customer::Email)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_customer_tenant")
                            .from(Customer::Table, Customer::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Customer::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Customer {
    Table,
    Id,
    Uuid,
    TenantId,
    Name,
    Email,
    PasswordHash,
    Cpf,
    Phone,
    MarketingConsent,
    ConsentAt,
    Active,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
