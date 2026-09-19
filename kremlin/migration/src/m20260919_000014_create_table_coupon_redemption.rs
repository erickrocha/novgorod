use crate::{
    m20260919_000002_create_table_customer::Customer,
    m20260919_000007_create_table_coupon::Coupon,
    m20260919_000011_create_table_orders::Orders,
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
                    .table(CouponRedemption::Table)
                    .if_not_exists()
                    .col(pk_auto(CouponRedemption::Id).big_integer())
                    .col(uuid_uniq(CouponRedemption::Uuid))
                    .col(big_integer(CouponRedemption::TenantId))
                    .col(big_integer(CouponRedemption::CouponId))
                    .col(big_integer(CouponRedemption::OrderId))
                    .col(big_integer(CouponRedemption::CustomerId))
                    .col(date_time(CouponRedemption::CreatedAt))
                    .col(string_len_null(CouponRedemption::CreatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_coupon_redemption_coupon_order")
                            .col(CouponRedemption::TenantId)
                            .col(CouponRedemption::CouponId)
                            .col(CouponRedemption::OrderId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_coupon_redemption_coupon")
                            .from_tbl(CouponRedemption::Table)
                            .from_col(CouponRedemption::TenantId)
                            .from_col(CouponRedemption::CouponId)
                            .to_tbl(Coupon::Table)
                            .to_col(Coupon::TenantId)
                            .to_col(Coupon::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_coupon_redemption_order")
                            .from_tbl(CouponRedemption::Table)
                            .from_col(CouponRedemption::TenantId)
                            .from_col(CouponRedemption::OrderId)
                            .to_tbl(Orders::Table)
                            .to_col(Orders::TenantId)
                            .to_col(Orders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_coupon_redemption_customer")
                            .from_tbl(CouponRedemption::Table)
                            .from_col(CouponRedemption::TenantId)
                            .from_col(CouponRedemption::CustomerId)
                            .to_tbl(Customer::Table)
                            .to_col(Customer::TenantId)
                            .to_col(Customer::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_coupon_redemption_tenant_coupon_customer")
                    .table(CouponRedemption::Table)
                    .col(CouponRedemption::TenantId)
                    .col(CouponRedemption::CouponId)
                    .col(CouponRedemption::CustomerId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CouponRedemption::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum CouponRedemption {
    Table,
    Id,
    Uuid,
    TenantId,
    CouponId,
    OrderId,
    CustomerId,
    CreatedAt,
    CreatedBy,
}
