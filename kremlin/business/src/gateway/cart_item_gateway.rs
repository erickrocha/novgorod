use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::cart_item::{CartItem, CartItemEntityMapper};
use entity::cart_item_entity;
use entity::prelude::CartItemEntity as CartItemQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter};

pub struct CartItemGateway {
    db: DbConn,
}

impl CartItemGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_cart_id(
        &self,
        cart_id: i64,
    ) -> Result<Vec<cart_item_entity::Model>, DbErr> {
        tenant_select(CartItemQuery::find(), cart_item_entity::Column::TenantId)
            .filter(cart_item_entity::Column::CartId.eq(cart_id))
            .all(&self.db)
            .await
    }
}

#[async_trait]
impl Gateway<CartItem, cart_item_entity::Model, cart_item_entity::ActiveModel> for CartItemGateway {
    async fn persist(&self, entity: CartItem) -> Result<cart_item_entity::ActiveModel, DbErr> {
        let active_model = CartItemEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CartItemQuery::delete_many().filter(cart_item_entity::Column::Id.eq(id)),
            cart_item_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<cart_item_entity::Model>, DbErr> {
        tenant_select(CartItemQuery::find(), cart_item_entity::Column::TenantId)
            .filter(cart_item_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<cart_item_entity::Model>, DbErr> {
        tenant_select(CartItemQuery::find(), cart_item_entity::Column::TenantId)
            .filter(cart_item_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<cart_item_entity::Model>, DbErr> {
        tenant_select(CartItemQuery::find(), cart_item_entity::Column::TenantId)
            .all(&self.db)
            .await
    }
}
