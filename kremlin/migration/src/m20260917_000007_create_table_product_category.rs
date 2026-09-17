use crate::{
    m20260917_000005_create_table_category::Category,
    m20260917_000006_create_table_product::Product,
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
                    .table(ProductCategory::Table)
                    .if_not_exists()
                    .col(pk_auto(ProductCategory::Id).big_integer())
                    .col(uuid_uniq(ProductCategory::Uuid))
                    .col(big_integer(ProductCategory::TenantId))
                    .col(big_integer(ProductCategory::ProductId))
                    .col(big_integer(ProductCategory::CategoryId))
                    .col(boolean(ProductCategory::IsPrimary).default(false))
                    .col(date_time(ProductCategory::CreatedAt))
                    .col(string_len_null(ProductCategory::CreatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_product_category_membership")
                            .col(ProductCategory::TenantId)
                            .col(ProductCategory::ProductId)
                            .col(ProductCategory::CategoryId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_category_product")
                            .from_tbl(ProductCategory::Table)
                            .from_col(ProductCategory::TenantId)
                            .from_col(ProductCategory::ProductId)
                            .to_tbl(Product::Table)
                            .to_col(Product::TenantId)
                            .to_col(Product::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_category_category")
                            .from_tbl(ProductCategory::Table)
                            .from_col(ProductCategory::TenantId)
                            .from_col(ProductCategory::CategoryId)
                            .to_tbl(Category::Table)
                            .to_col(Category::TenantId)
                            .to_col(Category::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProductCategory::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
pub enum ProductCategory {
    Table,
    Id,
    Uuid,
    TenantId,
    ProductId,
    CategoryId,
    IsPrimary,
    CreatedAt,
    CreatedBy,
}
