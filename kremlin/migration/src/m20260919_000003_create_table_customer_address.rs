use crate::m20260919_000002_create_table_customer::Customer;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CustomerAddress::Table)
                    .if_not_exists()
                    .col(pk_auto(CustomerAddress::Id).big_integer())
                    .col(uuid_uniq(CustomerAddress::Uuid))
                    .col(big_integer(CustomerAddress::TenantId))
                    .col(big_integer(CustomerAddress::CustomerId))
                    .col(string_len_null(CustomerAddress::Label, 40))
                    .col(string_len(CustomerAddress::Recipient, 150))
                    .col(string_len_null(CustomerAddress::AddressLine1, 500))
                    .col(string_len_null(CustomerAddress::AddressLine2, 500))
                    .col(string_len_null(CustomerAddress::Locality, 500))
                    .col(string_len_null(CustomerAddress::AdministrativeArea, 500))
                    .col(string_len_null(CustomerAddress::PostalCode, 20))
                    .col(string_len_null(CustomerAddress::CountryCode, 2))
                    .col(boolean(CustomerAddress::IsDefault).default(false))
                    .col(date_time(CustomerAddress::CreatedAt))
                    .col(string_len_null(CustomerAddress::CreatedBy, 255))
                    .col(date_time(CustomerAddress::UpdatedAt))
                    .col(string_len_null(CustomerAddress::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_customer_address_id")
                            .col(CustomerAddress::Id)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_customer_address_customer")
                            .from_tbl(CustomerAddress::Table)
                            .from_col(CustomerAddress::CustomerId)
                            .to_tbl(Customer::Table)
                            .to_col(Customer::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_customer_address_tenant_customer")
                    .table(CustomerAddress::Table)
                    .col(CustomerAddress::CustomerId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CustomerAddress::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum CustomerAddress {
    Table,
    Id,
    Uuid,
    TenantId,
    CustomerId,
    Label,
    Recipient,
    AddressLine1,
    AddressLine2,
    Locality,
    AdministrativeArea,
    PostalCode,
    CountryCode,
    IsDefault,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
