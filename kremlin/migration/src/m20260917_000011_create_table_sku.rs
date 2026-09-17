use crate::m20260917_000006_create_table_product::Product;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Sku::Table)
                    .if_not_exists()
                    .col(pk_auto(Sku::Id).big_integer())
                    .col(uuid_uniq(Sku::Uuid))
                    .col(big_integer(Sku::TenantId))
                    .col(big_integer(Sku::ProductId))
                    .col(string_len(Sku::Code, 64))
                    .col(string_len(Sku::VariantKey, 64))
                    .col(integer(Sku::PriceCents).check(Expr::cust("price_cents >= 0")))
                    .col(integer_null(Sku::CompareAtPriceCents).check(Expr::cust(
                        "compare_at_price_cents IS NULL OR compare_at_price_cents >= 0",
                    )))
                    .col(
                        integer_null(Sku::WeightG)
                            .check(Expr::cust("weight_g IS NULL OR weight_g >= 0")),
                    )
                    .col(
                        integer_null(Sku::WidthMm)
                            .check(Expr::cust("width_mm IS NULL OR width_mm >= 0")),
                    )
                    .col(
                        integer_null(Sku::HeightMm)
                            .check(Expr::cust("height_mm IS NULL OR height_mm >= 0")),
                    )
                    .col(
                        integer_null(Sku::LengthMm)
                            .check(Expr::cust("length_mm IS NULL OR length_mm >= 0")),
                    )
                    .col(boolean(Sku::Active).default(true))
                    .col(date_time(Sku::CreatedAt))
                    .col(string_len_null(Sku::CreatedBy, 255))
                    .col(date_time(Sku::UpdatedAt))
                    .col(string_len_null(Sku::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_sku_tenant_id_id_product")
                            .col(Sku::TenantId)
                            .col(Sku::Id)
                            .col(Sku::ProductId)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_sku_tenant_id_id")
                            .col(Sku::TenantId)
                            .col(Sku::Id)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_sku_tenant_code")
                            .col(Sku::TenantId)
                            .col(Sku::Code)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_sku_product_variant_key")
                            .col(Sku::TenantId)
                            .col(Sku::ProductId)
                            .col(Sku::VariantKey)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sku_product")
                            .from_tbl(Sku::Table)
                            .from_col(Sku::TenantId)
                            .from_col(Sku::ProductId)
                            .to_tbl(Product::Table)
                            .to_col(Product::TenantId)
                            .to_col(Product::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Sku::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
pub enum Sku {
    Table,
    Id,
    Uuid,
    TenantId,
    ProductId,
    Code,
    VariantKey,
    PriceCents,
    CompareAtPriceCents,
    WeightG,
    WidthMm,
    HeightMm,
    LengthMm,
    Active,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
