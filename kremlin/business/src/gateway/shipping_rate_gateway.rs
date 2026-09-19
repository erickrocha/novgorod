use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::shipping_rate::{ShippingRate, ShippingRateEntityMapper};
use entity::prelude::ShippingRateEntity as ShippingRateQuery;
use entity::shipping_rate_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct ShippingRateGateway {
    db: DbConn,
}

impl ShippingRateGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }
}

#[async_trait]
impl Gateway<ShippingRate, shipping_rate_entity::Model, shipping_rate_entity::ActiveModel>
    for ShippingRateGateway
{
    async fn persist(
        &self,
        entity: ShippingRate,
    ) -> Result<shipping_rate_entity::ActiveModel, DbErr> {
        let active_model = ShippingRateEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            ShippingRateQuery::delete_many().filter(shipping_rate_entity::Column::Id.eq(id)),
            shipping_rate_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<shipping_rate_entity::Model>, DbErr> {
        tenant_select(
            ShippingRateQuery::find(),
            shipping_rate_entity::Column::TenantId,
        )
        .filter(shipping_rate_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<shipping_rate_entity::Model>, DbErr> {
        tenant_select(
            ShippingRateQuery::find(),
            shipping_rate_entity::Column::TenantId,
        )
        .filter(shipping_rate_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<shipping_rate_entity::Model>, DbErr> {
        tenant_select(
            ShippingRateQuery::find(),
            shipping_rate_entity::Column::TenantId,
        )
        .order_by_asc(shipping_rate_entity::Column::Uf)
        .all(&self.db)
        .await
    }
}
