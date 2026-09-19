use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::order_status_history::{OrderStatusHistory, OrderStatusHistoryEntityMapper};
use entity::order_status_history_entity;
use entity::prelude::OrderStatusHistoryEntity as OrderStatusHistoryQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct OrderStatusHistoryGateway {
    db: DbConn,
}

impl OrderStatusHistoryGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_order_id(
        &self,
        order_id: i64,
    ) -> Result<Vec<order_status_history_entity::Model>, DbErr> {
        tenant_select(
            OrderStatusHistoryQuery::find(),
            order_status_history_entity::Column::TenantId,
        )
        .filter(order_status_history_entity::Column::OrderId.eq(order_id))
        .order_by_asc(order_status_history_entity::Column::CreatedAt)
        .all(&self.db)
        .await
    }
}

#[async_trait]
impl Gateway<OrderStatusHistory, order_status_history_entity::Model, order_status_history_entity::ActiveModel>
    for OrderStatusHistoryGateway
{
    async fn persist(
        &self,
        entity: OrderStatusHistory,
    ) -> Result<order_status_history_entity::ActiveModel, DbErr> {
        let active_model = OrderStatusHistoryEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            OrderStatusHistoryQuery::delete_many()
                .filter(order_status_history_entity::Column::Id.eq(id)),
            order_status_history_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<order_status_history_entity::Model>, DbErr> {
        tenant_select(
            OrderStatusHistoryQuery::find(),
            order_status_history_entity::Column::TenantId,
        )
        .filter(order_status_history_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<order_status_history_entity::Model>, DbErr> {
        tenant_select(
            OrderStatusHistoryQuery::find(),
            order_status_history_entity::Column::TenantId,
        )
        .filter(order_status_history_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<order_status_history_entity::Model>, DbErr> {
        tenant_select(
            OrderStatusHistoryQuery::find(),
            order_status_history_entity::Column::TenantId,
        )
        .order_by_asc(order_status_history_entity::Column::CreatedAt)
        .all(&self.db)
        .await
    }
}
