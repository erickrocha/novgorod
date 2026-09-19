use crate::{
    m20260917_000011_create_table_sku::Sku,
    m20260919_000011_create_table_orders::Orders,
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
                    .table(OrderItem::Table)
                    .if_not_exists()
                    .col(pk_auto(OrderItem::Id).big_integer())
                    .col(uuid_uniq(OrderItem::Uuid))
                    .col(big_integer(OrderItem::TenantId))
                    .col(big_integer(OrderItem::OrderId))
                    .col(big_integer(OrderItem::SkuId))
                    .col(string_len(OrderItem::SkuCode, 64))
                    .col(string_len(OrderItem::ProductName, 200))
                    .col(string_len_null(OrderItem::AttributesDesc, 200))
                    .col(integer(OrderItem::Quantity).check(Expr::cust("quantity > 0")))
                    .col(integer(OrderItem::UnitPriceCents).check(Expr::cust("unit_price_cents >= 0")))
                    .col(integer(OrderItem::DiscountCents).default(0).check(Expr::cust("discount_cents >= 0")))
                    .col(string_len(OrderItem::Ncm, 8))
                    .col(string_len(OrderItem::Cfop, 4))
                    .col(string_len(OrderItem::Csosn, 4))
                    .col(integer(OrderItem::IcmsRateBp).default(0))
                    .col(integer(OrderItem::TaxCents).default(0))
                    .col(integer(OrderItem::TotalCents).check(Expr::cust("total_cents >= 0")))
                    .col(date_time(OrderItem::CreatedAt))
                    .col(string_len_null(OrderItem::CreatedBy, 255))
                    .col(date_time(OrderItem::UpdatedAt))
                    .col(string_len_null(OrderItem::UpdatedBy, 255))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_item_order")
                            .from_tbl(OrderItem::Table)
                            .from_col(OrderItem::TenantId)
                            .from_col(OrderItem::OrderId)
                            .to_tbl(Orders::Table)
                            .to_col(Orders::TenantId)
                            .to_col(Orders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_item_sku")
                            .from_tbl(OrderItem::Table)
                            .from_col(OrderItem::TenantId)
                            .from_col(OrderItem::SkuId)
                            .to_tbl(Sku::Table)
                            .to_col(Sku::TenantId)
                            .to_col(Sku::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_order_item_tenant_order")
                    .table(OrderItem::Table)
                    .col(OrderItem::TenantId)
                    .col(OrderItem::OrderId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrderItem::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum OrderItem {
    Table,
    Id,
    Uuid,
    TenantId,
    OrderId,
    SkuId,
    SkuCode,
    ProductName,
    AttributesDesc,
    Quantity,
    UnitPriceCents,
    DiscountCents,
    Ncm,
    Cfop,
    Csosn,
    IcmsRateBp,
    TaxCents,
    TotalCents,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
