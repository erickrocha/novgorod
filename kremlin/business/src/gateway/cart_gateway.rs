use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::cart::{Cart, CartEntityMapper};
use entity::cart_entity;
use entity::prelude::CartEntity as CartQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter};

pub struct CartGateway {
    db: DbConn,
}

impl CartGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }
}

#[async_trait]
impl Gateway<Cart, cart_entity::Model, cart_entity::ActiveModel> for CartGateway {
    async fn persist(&self, entity: Cart) -> Result<cart_entity::ActiveModel, DbErr> {
        let active_model = CartEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CartQuery::delete_many().filter(cart_entity::Column::Id.eq(id)),
            cart_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<cart_entity::Model>, DbErr> {
        tenant_select(CartQuery::find(), cart_entity::Column::TenantId)
            .filter(cart_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<cart_entity::Model>, DbErr> {
        tenant_select(CartQuery::find(), cart_entity::Column::TenantId)
            .filter(cart_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<cart_entity::Model>, DbErr> {
        tenant_select(CartQuery::find(), cart_entity::Column::TenantId)
            .all(&self.db)
            .await
    }
}
