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
                    .table(Campaign::Table)
                    .if_not_exists()
                    .col(pk_auto(Campaign::Id).big_integer())
                    .col(uuid_uniq(Campaign::Uuid))
                    .col(big_integer(Campaign::TenantId))
                    .col(string_len(Campaign::Name, 150))
                    .col(string_len(Campaign::Type, 20))
                    .col(integer(Campaign::Value))
                    .col(string_len(Campaign::Scope, 20))
                    .col(date_time(Campaign::StartsAt))
                    .col(date_time(Campaign::EndsAt))
                    .col(boolean(Campaign::Active).default(true))
                    .col(date_time(Campaign::CreatedAt))
                    .col(string_len_null(Campaign::CreatedBy, 255))
                    .col(date_time(Campaign::UpdatedAt))
                    .col(string_len_null(Campaign::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_campaign_tenant_id_id")
                            .col(Campaign::TenantId)
                            .col(Campaign::Id)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_campaign_tenant")
                            .from(Campaign::Table, Campaign::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_campaign_tenant_active_period")
                    .table(Campaign::Table)
                    .col(Campaign::TenantId)
                    .col(Campaign::Active)
                    .col(Campaign::StartsAt)
                    .col(Campaign::EndsAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Campaign::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Campaign {
    Table,
    Id,
    Uuid,
    TenantId,
    Name,
    #[sea_orm(iden = "campaign_type")]
    Type,
    Value,
    Scope,
    StartsAt,
    EndsAt,
    Active,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
