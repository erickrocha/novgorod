use crate::{
    m20260917_000006_create_table_product::Product, m20260917_000011_create_table_sku::Sku,
};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(ProductImage::Table).if_not_exists()
            .col(pk_auto(ProductImage::Id))
            .col(binary_len_uniq(ProductImage::Uuid, 16))
            .col(integer(ProductImage::TenantId))
            .col(integer(ProductImage::ProductId))
            .col(integer_null(ProductImage::SkuId))
            .col(string_len_null(ProductImage::AltText, 255))
            .col(integer(ProductImage::SortOrder).default(0).check(Expr::cust("sort_order >= 0")))
            .col(boolean(ProductImage::IsPrimary).default(false).check(Expr::cust("NOT is_primary OR sku_id IS NULL")))
            .col(string_len(ProductImage::StorageProvider, 32))
            .col(string_len(ProductImage::Bucket, 255))
            .col(string_len(ProductImage::ObjectKey, 1024))
            .col(binary_len_uniq(ProductImage::StorageIdentityHash, 32))
            .col(string_len_null(ProductImage::ObjectVersion, 255))
            .col(string_len_null(ProductImage::Etag, 255))
            .col(binary_len_null(ProductImage::ChecksumSha256, 32))
            .col(string_len(ProductImage::OriginalFilename, 255))
            .col(string_len(ProductImage::MimeType, 127))
            .col(big_integer(ProductImage::SizeBytes).check(Expr::cust("size_bytes >= 0")))
            .col(integer_null(ProductImage::WidthPx).check(Expr::cust("width_px IS NULL OR width_px > 0")))
            .col(integer_null(ProductImage::HeightPx).check(Expr::cust("height_px IS NULL OR height_px > 0")))
            .col(string_len(ProductImage::StorageStatus, 20).default("pending").check(Expr::cust("storage_status IN ('pending','available','delete_pending','deleted','failed')")))
            .col(date_time(ProductImage::CreatedAt))
            .col(string_len_null(ProductImage::CreatedBy, 255))
            .col(date_time(ProductImage::UpdatedAt))
            .col(string_len_null(ProductImage::UpdatedBy, 255))
            .col(date_time_null(ProductImage::DeletedAt))
            .col(string_len_null(ProductImage::DeletedBy, 255))
            .foreign_key(ForeignKey::create().name("fk_product_image_product").from_tbl(ProductImage::Table).from_col(ProductImage::TenantId).from_col(ProductImage::ProductId).to_tbl(Product::Table).to_col(Product::TenantId).to_col(Product::Id).on_delete(ForeignKeyAction::Restrict))
            .foreign_key(ForeignKey::create().name("fk_product_image_sku").from_tbl(ProductImage::Table).from_col(ProductImage::TenantId).from_col(ProductImage::SkuId).from_col(ProductImage::ProductId).to_tbl(Sku::Table).to_col(Sku::TenantId).to_col(Sku::Id).to_col(Sku::ProductId).on_delete(ForeignKeyAction::Restrict))
            .to_owned()).await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_product_image_gallery")
                    .table(ProductImage::Table)
                    .col(ProductImage::TenantId)
                    .col(ProductImage::ProductId)
                    .col(ProductImage::SortOrder)
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProductImage::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
pub enum ProductImage {
    Table,
    Id,
    Uuid,
    TenantId,
    ProductId,
    SkuId,
    AltText,
    SortOrder,
    IsPrimary,
    StorageProvider,
    Bucket,
    ObjectKey,
    StorageIdentityHash,
    ObjectVersion,
    Etag,
    ChecksumSha256,
    OriginalFilename,
    MimeType,
    SizeBytes,
    WidthPx,
    HeightPx,
    StorageStatus,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
    DeletedAt,
    DeletedBy,
}
