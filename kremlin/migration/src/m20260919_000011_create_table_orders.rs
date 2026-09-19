use crate::{
    m20260917_000002_create_tenant_table::Tenant,
    m20260919_000002_create_table_customer::Customer,
    m20260919_000007_create_table_coupon::Coupon,
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
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(pk_auto(Orders::Id).big_integer())
                    .col(uuid_uniq(Orders::Uuid))
                    .col(big_integer(Orders::TenantId))
                    .col(string_len(Orders::Number, 20))
                    .col(big_integer(Orders::CustomerId))
                    .col(string_len(Orders::Status, 30))
                    .col(string_len(Orders::PaymentStatus, 30))
                    .col(integer(Orders::SubtotalCents).check(Expr::cust("subtotal_cents >= 0")))
                    .col(integer(Orders::DiscountCents).default(0).check(Expr::cust("discount_cents >= 0")))
                    .col(integer(Orders::ShippingCents).default(0).check(Expr::cust("shipping_cents >= 0")))
                    .col(integer(Orders::TaxTotalCents).default(0).check(Expr::cust("tax_total_cents >= 0")))
                    .col(integer(Orders::TotalCents).check(Expr::cust("total_cents >= 0")))
                    .col(big_integer_null(Orders::CouponId))
                    .col(string_len_null(Orders::CouponCode, 40))
                    .col(string_len(Orders::ShipRecipient, 150))
                    .col(string_len(Orders::ShipCep, 8))
                    .col(string_len(Orders::ShipLogradouro, 200))
                    .col(string_len(Orders::ShipNumero, 20))
                    .col(string_len_null(Orders::ShipComplemento, 100))
                    .col(string_len(Orders::ShipBairro, 100))
                    .col(string_len(Orders::ShipCidade, 100))
                    .col(string_len(Orders::ShipUf, 2))
                    .col(date_time(Orders::PlacedAt))
                    .col(date_time(Orders::CreatedAt))
                    .col(string_len_null(Orders::CreatedBy, 255))
                    .col(date_time(Orders::UpdatedAt))
                    .col(string_len_null(Orders::UpdatedBy, 255))
                    .index(
                        Index::create()
                            .name("uq_orders_tenant_id_id")
                            .col(Orders::TenantId)
                            .col(Orders::Id)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("uq_orders_tenant_number")
                            .col(Orders::TenantId)
                            .col(Orders::Number)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_tenant")
                            .from(Orders::Table, Orders::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_customer")
                            .from_tbl(Orders::Table)
                            .from_col(Orders::TenantId)
                            .from_col(Orders::CustomerId)
                            .to_tbl(Customer::Table)
                            .to_col(Customer::TenantId)
                            .to_col(Customer::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_coupon")
                            .from_tbl(Orders::Table)
                            .from_col(Orders::TenantId)
                            .from_col(Orders::CouponId)
                            .to_tbl(Coupon::Table)
                            .to_col(Coupon::TenantId)
                            .to_col(Coupon::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_orders_tenant_customer_status")
                    .table(Orders::Table)
                    .col(Orders::TenantId)
                    .col(Orders::CustomerId)
                    .col(Orders::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_orders_tenant_created_at")
                    .table(Orders::Table)
                    .col(Orders::TenantId)
                    .col(Orders::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Orders::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Orders {
    Table,
    Id,
    Uuid,
    TenantId,
    Number,
    CustomerId,
    Status,
    PaymentStatus,
    SubtotalCents,
    DiscountCents,
    ShippingCents,
    TaxTotalCents,
    TotalCents,
    CouponId,
    CouponCode,
    ShipRecipient,
    ShipCep,
    ShipLogradouro,
    ShipNumero,
    ShipComplemento,
    ShipBairro,
    ShipCidade,
    ShipUf,
    PlacedAt,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    UpdatedBy,
}
