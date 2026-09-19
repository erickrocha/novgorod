use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::orders::{Orders, OrdersEntityMapper};
use entity::orders_entity;
use entity::prelude::OrdersEntity as OrdersQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct OrdersGateway {
    db: DbConn,
}

impl OrdersGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_number(&self, number: String) -> Result<Option<orders_entity::Model>, DbErr> {
        tenant_select(OrdersQuery::find(), orders_entity::Column::TenantId)
            .filter(orders_entity::Column::Number.eq(number))
            .one(&self.db)
            .await
    }

    pub async fn find_by_customer_id(
        &self,
        customer_id: i64,
    ) -> Result<Vec<orders_entity::Model>, DbErr> {
        tenant_select(OrdersQuery::find(), orders_entity::Column::TenantId)
            .filter(orders_entity::Column::CustomerId.eq(customer_id))
            .order_by_desc(orders_entity::Column::PlacedAt)
            .all(&self.db)
            .await
    }
}

#[async_trait]
impl Gateway<Orders, orders_entity::Model, orders_entity::ActiveModel> for OrdersGateway {
    async fn persist(&self, entity: Orders) -> Result<orders_entity::ActiveModel, DbErr> {
        let active_model = OrdersEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            OrdersQuery::delete_many().filter(orders_entity::Column::Id.eq(id)),
            orders_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<orders_entity::Model>, DbErr> {
        tenant_select(OrdersQuery::find(), orders_entity::Column::TenantId)
            .filter(orders_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<orders_entity::Model>, DbErr> {
        tenant_select(OrdersQuery::find(), orders_entity::Column::TenantId)
            .filter(orders_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<orders_entity::Model>, DbErr> {
        tenant_select(OrdersQuery::find(), orders_entity::Column::TenantId)
            .order_by_desc(orders_entity::Column::PlacedAt)
            .all(&self.db)
            .await
    }
}
