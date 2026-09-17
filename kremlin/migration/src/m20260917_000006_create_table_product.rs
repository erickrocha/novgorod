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
                    .table(Product::Table)
                    .if_not_exists()
                    .col(pk_auto(Product::Id).big_integer())
                    .col(uuid_uniq(Product::Uuid))
                    .col(big_integer(Product::TenantId))
                    .col(string_len(Product::Name, 200))
                    .col(string_len(Product::Slug, 220))
                    .col(text_null(Product::Description))
                    .col(string_len_null(Product::Brand, 120))
                    .col(boolean(Product::Active).default(true))
                    .col(string_len(Product::Ncm, 8).check(Expr::cust("CHAR_LENGTH(ncm) = 8")))
                    .col(
                        string_len_null(Product::Cest, 7)
                            .check(Expr::cust("cest IS NULL OR CHAR_LENGTH(cest) = 7")),
                    )
                    .col(
                        tiny_integer(Product::OrigemMercadoria)
                            .check(Expr::cust("origem_mercadoria BETWEEN 0 AND 8")),
                    )
                    .col(date_time(Product::CreatedAt))
                    .col(string_len_null(Product::CreatedBy, 255))
                    .col(date_time(Product::UpdatedAt))
                    .col(string_len_null(Product::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_product_tenant_id_id")
                            .col(Product::TenantId)
                            .col(Product::Id)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_product_tenant_slug")
                            .col(Product::TenantId)
                            .col(Product::Slug)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_tenant")
                            .from(Product::Table, Product::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Product::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
pub enum Product {
    Table,
    Id,
    Uuid,
    TenantId,
    Name,
    Slug,
    Description,
    Brand,
    Active,
    Ncm,
    Cest,
    OrigemMercadoria,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
