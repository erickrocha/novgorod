use crate::m20260917_000008_create_table_catalog_attribute::CatalogAttribute;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CatalogAttributeValue::Table)
                    .if_not_exists()
                    .col(pk_auto(CatalogAttributeValue::Id))
                    .col(uuid_uniq(CatalogAttributeValue::Uuid))
                    .col(integer(CatalogAttributeValue::TenantId))
                    .col(integer(CatalogAttributeValue::AttributeId))
                    .col(string_len(CatalogAttributeValue::Value, 80))
                    .col(date_time(CatalogAttributeValue::CreatedAt))
                    .col(string_len_null(CatalogAttributeValue::CreatedBy, 255))
                    .col(date_time(CatalogAttributeValue::UpdatedAt))
                    .col(string_len_null(CatalogAttributeValue::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_catalog_attribute_value_tenant_id_id_attribute")
                            .col(CatalogAttributeValue::TenantId)
                            .col(CatalogAttributeValue::Id)
                            .col(CatalogAttributeValue::AttributeId)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_catalog_attribute_value")
                            .col(CatalogAttributeValue::TenantId)
                            .col(CatalogAttributeValue::AttributeId)
                            .col(CatalogAttributeValue::Value)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_catalog_attribute_value_attribute")
                            .from_tbl(CatalogAttributeValue::Table)
                            .from_col(CatalogAttributeValue::TenantId)
                            .from_col(CatalogAttributeValue::AttributeId)
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
            .drop_table(Table::drop().table(CatalogAttributeValue::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
pub enum CatalogAttributeValue {
    Table,
    Id,
    Uuid,
    TenantId,
    AttributeId,
    Value,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
