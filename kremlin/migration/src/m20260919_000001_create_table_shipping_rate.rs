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
                    .table(ShippingRate::Table)
                    .if_not_exists()
                    .col(pk_auto(ShippingRate::Id).big_integer())
                    .col(uuid_uniq(ShippingRate::Uuid))
                    .col(big_integer(ShippingRate::TenantId))
                    .col(string_len(ShippingRate::Uf, 2))
                    .col(integer(ShippingRate::PriceCents).check(Expr::cust("price_cents >= 0")))
                    .col(date_time(ShippingRate::CreatedAt))
                    .col(string_len_null(ShippingRate::CreatedBy, 255))
                    .col(date_time(ShippingRate::UpdatedAt))
                    .col(string_len_null(ShippingRate::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_shipping_rate_tenant_id_id")
                            .col(ShippingRate::TenantId)
                            .col(ShippingRate::Id)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_shipping_rate_tenant_uf")
                            .col(ShippingRate::TenantId)
                            .col(ShippingRate::Uf)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_shipping_rate_tenant")
                            .from(ShippingRate::Table, ShippingRate::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ShippingRate::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ShippingRate {
    Table,
    Id,
    Uuid,
    TenantId,
    Uf,
    PriceCents,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
