use crate::m20260919_000011_create_table_orders::Orders;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OrderStatusHistory::Table)
                    .if_not_exists()
                    .col(pk_auto(OrderStatusHistory::Id).big_integer())
                    .col(uuid_uniq(OrderStatusHistory::Uuid))
                    .col(big_integer(OrderStatusHistory::TenantId))
                    .col(big_integer(OrderStatusHistory::OrderId))
                    .col(string_len_null(OrderStatusHistory::FromStatus, 30))
                    .col(string_len(OrderStatusHistory::ToStatus, 30))
                    .col(string_len(OrderStatusHistory::ActorType, 20))
                    .col(big_integer_null(OrderStatusHistory::ActorId))
                    .col(text_null(OrderStatusHistory::Note))
                    .col(date_time(OrderStatusHistory::CreatedAt))
                    .col(string_len_null(OrderStatusHistory::CreatedBy, 255))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_order_status_history_order")
                            .from_tbl(OrderStatusHistory::Table)
                            .from_col(OrderStatusHistory::TenantId)
                            .from_col(OrderStatusHistory::OrderId)
                            .to_tbl(Orders::Table)
                            .to_col(Orders::TenantId)
                            .to_col(Orders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_order_status_history_tenant_order")
                    .table(OrderStatusHistory::Table)
                    .col(OrderStatusHistory::TenantId)
                    .col(OrderStatusHistory::OrderId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrderStatusHistory::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum OrderStatusHistory {
    Table,
    Id,
    Uuid,
    TenantId,
    OrderId,
    FromStatus,
    ToStatus,
    ActorType,
    ActorId,
    Note,
    CreatedAt,
    CreatedBy,
}
