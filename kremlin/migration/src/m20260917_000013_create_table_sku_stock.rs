use crate::m20260917_000011_create_table_sku::Sku;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SkuStock::Table)
                    .if_not_exists()
                    .col(pk_auto(SkuStock::Id).big_integer())
                    .col(uuid_uniq(SkuStock::Uuid))
                    .col(big_integer(SkuStock::TenantId))
                    .col(big_integer(SkuStock::SkuId))
                    .col(
                        integer(SkuStock::Quantity)
                            .default(0)
                            .check(Expr::cust("quantity >= 0")),
                    )
                    .col(
                        integer(SkuStock::Reserved)
                            .default(0)
                            .check(Expr::cust("reserved >= 0 AND reserved <= quantity")),
                    )
                    .col(date_time(SkuStock::CreatedAt))
                    .col(string_len_null(SkuStock::CreatedBy, 255))
                    .col(date_time(SkuStock::UpdatedAt))
                    .col(string_len_null(SkuStock::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_sku_stock_tenant_sku")
                            .col(SkuStock::TenantId)
                            .col(SkuStock::SkuId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sku_stock_sku")
                            .from_tbl(SkuStock::Table)
                            .from_col(SkuStock::TenantId)
                            .from_col(SkuStock::SkuId)
                            .to_tbl(Sku::Table)
                            .to_col(Sku::TenantId)
                            .to_col(Sku::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SkuStock::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
pub enum SkuStock {
    Table,
    Id,
    Uuid,
    TenantId,
    SkuId,
    Quantity,
    Reserved,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
