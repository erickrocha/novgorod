use crate::{
    m20260917_000002_create_tenant_table::Tenant,
    m20260919_000002_create_table_customer::Customer,
};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Cart::Table)
                    .if_not_exists()
                    .col(pk_auto(Cart::Id).big_integer())
                    .col(uuid_uniq(Cart::Uuid))
                    .col(big_integer(Cart::TenantId))
                    .col(big_integer_null(Cart::CustomerId))
                    .col(string_len(Cart::Status, 20))
                    .col(date_time_null(Cart::ExpiresAt))
                    .col(date_time(Cart::CreatedAt))
                    .col(string_len_null(Cart::CreatedBy, 255))
                    .col(date_time(Cart::UpdatedAt))
                    .col(string_len_null(Cart::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_cart_tenant_id_id")
                            .col(Cart::TenantId)
                            .col(Cart::Id)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cart_tenant")
                            .from(Cart::Table, Cart::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cart_customer")
                            .from_tbl(Cart::Table)
                            .from_col(Cart::TenantId)
                            .from_col(Cart::CustomerId)
                            .to_tbl(Customer::Table)
                            .to_col(Customer::TenantId)
                            .to_col(Customer::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cart_tenant_customer")
                    .table(Cart::Table)
                    .col(Cart::TenantId)
                    .col(Cart::CustomerId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Cart::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Cart {
    Table,
    Id,
    Uuid,
    TenantId,
    CustomerId,
    Status,
    ExpiresAt,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
