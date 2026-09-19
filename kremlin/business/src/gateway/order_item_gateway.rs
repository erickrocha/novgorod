use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::order_item::{OrderItem, OrderItemEntityMapper};
use entity::order_item_entity;
use entity::prelude::OrderItemEntity as OrderItemQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter};

pub struct OrderItemGateway {
    db: DbConn,
}

impl OrderItemGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_order_id(
        &self,
        order_id: i64,
    ) -> Result<Vec<order_item_entity::Model>, DbErr> {
        tenant_select(OrderItemQuery::find(), order_item_entity::Column::TenantId)
            .filter(order_item_entity::Column::OrderId.eq(order_id))
            .all(&self.db)
            .await
    }
}

#[async_trait]
impl Gateway<OrderItem, order_item_entity::Model, order_item_entity::ActiveModel>
    for OrderItemGateway
{
    async fn persist(&self, entity: OrderItem) -> Result<order_item_entity::ActiveModel, DbErr> {
        let active_model = OrderItemEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            OrderItemQuery::delete_many().filter(order_item_entity::Column::Id.eq(id)),
            order_item_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<order_item_entity::Model>, DbErr> {
        tenant_select(OrderItemQuery::find(), order_item_entity::Column::TenantId)
            .filter(order_item_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<order_item_entity::Model>, DbErr> {
        tenant_select(OrderItemQuery::find(), order_item_entity::Column::TenantId)
            .filter(order_item_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<order_item_entity::Model>, DbErr> {
        tenant_select(OrderItemQuery::find(), order_item_entity::Column::TenantId)
            .all(&self.db)
            .await
    }
}
