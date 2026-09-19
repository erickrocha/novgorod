use crate::{
    m20260917_000011_create_table_sku::Sku,
    m20260919_000009_create_table_cart::Cart,
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
                    .table(CartItem::Table)
                    .if_not_exists()
                    .col(pk_auto(CartItem::Id).big_integer())
                    .col(uuid_uniq(CartItem::Uuid))
                    .col(big_integer(CartItem::TenantId))
                    .col(big_integer(CartItem::CartId))
                    .col(big_integer(CartItem::SkuId))
                    .col(integer(CartItem::Quantity).check(Expr::cust("quantity > 0")))
                    .col(integer(CartItem::UnitPriceCents).check(Expr::cust("unit_price_cents >= 0")))
                    .col(date_time(CartItem::CreatedAt))
                    .col(string_len_null(CartItem::CreatedBy, 255))
                    .col(date_time(CartItem::UpdatedAt))
                    .col(string_len_null(CartItem::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_cart_item_cart_sku")
                            .col(CartItem::TenantId)
                            .col(CartItem::CartId)
                            .col(CartItem::SkuId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cart_item_cart")
                            .from_tbl(CartItem::Table)
                            .from_col(CartItem::TenantId)
                            .from_col(CartItem::CartId)
                            .to_tbl(Cart::Table)
                            .to_col(Cart::TenantId)
                            .to_col(Cart::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cart_item_sku")
                            .from_tbl(CartItem::Table)
                            .from_col(CartItem::TenantId)
                            .from_col(CartItem::SkuId)
                            .to_tbl(Sku::Table)
                            .to_col(Sku::TenantId)
                            .to_col(Sku::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CartItem::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum CartItem {
    Table,
    Id,
    Uuid,
    TenantId,
    CartId,
    SkuId,
    Quantity,
    UnitPriceCents,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
