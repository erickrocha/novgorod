use crate::{
    m20260917_000006_create_table_product::Product,
    m20260917_000008_create_table_catalog_attribute::CatalogAttribute,
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
                    .table(ProductAttribute::Table)
                    .if_not_exists()
                    .col(pk_auto(ProductAttribute::Id).big_integer())
                    .col(uuid_uniq(ProductAttribute::Uuid))
                    .col(big_integer(ProductAttribute::TenantId))
                    .col(big_integer(ProductAttribute::ProductId))
                    .col(big_integer(ProductAttribute::AttributeId))
                    .col(boolean(ProductAttribute::Required).default(false))
                    .col(
                        integer(ProductAttribute::SortOrder)
                            .default(0)
                            .check(Expr::cust("sort_order >= 0")),
                    )
                    .col(date_time(ProductAttribute::CreatedAt))
                    .col(string_len_null(ProductAttribute::CreatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_product_attribute_tenant_id_id_attribute_product")
                            .col(ProductAttribute::TenantId)
                            .col(ProductAttribute::Id)
                            .col(ProductAttribute::AttributeId)
                            .col(ProductAttribute::ProductId)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_product_attribute_membership")
                            .col(ProductAttribute::TenantId)
                            .col(ProductAttribute::ProductId)
                            .col(ProductAttribute::AttributeId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_attribute_product")
                            .from_tbl(ProductAttribute::Table)
                            .from_col(ProductAttribute::TenantId)
                            .from_col(ProductAttribute::ProductId)
                            .to_tbl(Product::Table)
                            .to_col(Product::TenantId)
                            .to_col(Product::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_attribute_attribute")
                            .from_tbl(ProductAttribute::Table)
                            .from_col(ProductAttribute::TenantId)
                            .from_col(ProductAttribute::AttributeId)
                            .to_tbl(CatalogAttribute::Table)
                            .to_col(CatalogAttribute::TenantId)
                            .to_col(CatalogAttribute::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProductAttribute::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
pub enum ProductAttribute {
    Table,
    Id,
    Uuid,
    TenantId,
    ProductId,
    AttributeId,
    Required,
    SortOrder,
    CreatedAt,
    CreatedBy,
}
