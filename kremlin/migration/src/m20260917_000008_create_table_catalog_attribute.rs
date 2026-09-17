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
                    .table(CatalogAttribute::Table)
                    .if_not_exists()
                    .col(pk_auto(CatalogAttribute::Id).big_integer())
                    .col(uuid_uniq(CatalogAttribute::Uuid))
                    .col(big_integer(CatalogAttribute::TenantId))
                    .col(string_len(CatalogAttribute::Name, 80))
                    .col(string_len(CatalogAttribute::DisplayType, 20))
                    .col(date_time(CatalogAttribute::CreatedAt))
                    .col(string_len_null(CatalogAttribute::CreatedBy, 255))
                    .col(date_time(CatalogAttribute::UpdatedAt))
                    .col(string_len_null(CatalogAttribute::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_catalog_attribute_tenant_id_id")
                            .col(CatalogAttribute::TenantId)
                            .col(CatalogAttribute::Id)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_catalog_attribute_tenant_name")
                            .col(CatalogAttribute::TenantId)
                            .col(CatalogAttribute::Name)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_catalog_attribute_tenant")
                            .from(CatalogAttribute::Table, CatalogAttribute::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CatalogAttribute::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
pub enum CatalogAttribute {
    Table,
    Id,
    Uuid,
    TenantId,
    Name,
    DisplayType,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
