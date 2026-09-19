use crate::m20260919_000005_create_table_campaign::Campaign;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CampaignTarget::Table)
                    .if_not_exists()
                    .col(pk_auto(CampaignTarget::Id).big_integer())
                    .col(uuid_uniq(CampaignTarget::Uuid))
                    .col(big_integer(CampaignTarget::TenantId))
                    .col(big_integer(CampaignTarget::CampaignId))
                    .col(string_len(CampaignTarget::TargetType, 20))
                    .col(big_integer(CampaignTarget::TargetId))
                    .col(date_time(CampaignTarget::CreatedAt))
                    .col(string_len_null(CampaignTarget::CreatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_campaign_target_membership")
                            .col(CampaignTarget::TenantId)
                            .col(CampaignTarget::CampaignId)
                            .col(CampaignTarget::TargetType)
                            .col(CampaignTarget::TargetId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_campaign_target_campaign")
                            .from_tbl(CampaignTarget::Table)
                            .from_col(CampaignTarget::TenantId)
                            .from_col(CampaignTarget::CampaignId)
                            .to_tbl(Campaign::Table)
                            .to_col(Campaign::TenantId)
                            .to_col(Campaign::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CampaignTarget::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum CampaignTarget {
    Table,
    Id,
    Uuid,
    TenantId,
    CampaignId,
    TargetType,
    TargetId,
    CreatedAt,
    CreatedBy,
}
