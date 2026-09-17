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
                    .table(Category::Table)
                    .if_not_exists()
                    .col(pk_auto(Category::Id))
                    .col(uuid_uniq(Category::Uuid))
                    .col(integer(Category::TenantId))
                    .col(string_len(Category::Name, 120))
                    .col(string_len(Category::Slug, 140))
                    .col(integer_null(Category::ParentId))
                    .col(boolean(Category::Active).default(true))
                    .col(date_time(Category::CreatedAt))
                    .col(string_len_null(Category::CreatedBy, 255))
                    .col(date_time(Category::UpdatedAt))
                    .col(string_len_null(Category::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_category_tenant_id_id")
                            .col(Category::TenantId)
                            .col(Category::Id)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_category_tenant_slug")
                            .col(Category::TenantId)
                            .col(Category::Slug)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_category_tenant")
                            .from(Category::Table, Category::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_category_parent")
                            .from_tbl(Category::Table)
                            .from_col(Category::TenantId)
                            .from_col(Category::ParentId)
                            .to_tbl(Category::Table)
                            .to_col(Category::TenantId)
                            .to_col(Category::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Category::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Category {
    Table,
    Id,
    Uuid,
    TenantId,
    Name,
    Slug,
    ParentId,
    Active,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
