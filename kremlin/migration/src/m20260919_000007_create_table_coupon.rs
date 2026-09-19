use crate::{
    m20260917_000002_create_tenant_table::Tenant,
    m20260919_000005_create_table_campaign::Campaign,
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
                    .table(Coupon::Table)
                    .if_not_exists()
                    .col(pk_auto(Coupon::Id).big_integer())
                    .col(uuid_uniq(Coupon::Uuid))
                    .col(big_integer(Coupon::TenantId))
                    .col(string_len(Coupon::Code, 40))
                    .col(big_integer_null(Coupon::CampaignId))
                    .col(string_len(Coupon::Type, 20))
                    .col(integer(Coupon::Value))
                    .col(integer_null(Coupon::MinOrderCents))
                    .col(integer_null(Coupon::MaxUses))
                    .col(integer_null(Coupon::MaxUsesPerCustomer))
                    .col(date_time_null(Coupon::StartsAt))
                    .col(date_time_null(Coupon::ExpiresAt))
                    .col(boolean(Coupon::Active).default(true))
                    .col(date_time(Coupon::CreatedAt))
                    .col(string_len_null(Coupon::CreatedBy, 255))
                    .col(date_time(Coupon::UpdatedAt))
                    .col(string_len_null(Coupon::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_coupon_tenant_id_id")
                            .col(Coupon::TenantId)
                            .col(Coupon::Id)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_coupon_tenant_code")
                            .col(Coupon::TenantId)
                            .col(Coupon::Code)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_coupon_tenant")
                            .from(Coupon::Table, Coupon::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_coupon_campaign")
                            .from_tbl(Coupon::Table)
                            .from_col(Coupon::TenantId)
                            .from_col(Coupon::CampaignId)
                            .to_tbl(Campaign::Table)
                            .to_col(Campaign::TenantId)
                            .to_col(Campaign::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_coupon_tenant_active_expires")
                    .table(Coupon::Table)
                    .col(Coupon::TenantId)
                    .col(Coupon::Active)
                    .col(Coupon::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Coupon::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Coupon {
    Table,
    Id,
    Uuid,
    TenantId,
    Code,
    CampaignId,
    #[sea_orm(iden = "coupon_type")]
    Type,
    Value,
    MinOrderCents,
    MaxUses,
    MaxUsesPerCustomer,
    StartsAt,
    ExpiresAt,
    Active,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
