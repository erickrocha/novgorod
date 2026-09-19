use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::product::{Product, ProductEntityMapper};
use entity::prelude::ProductEntity as ProductQuery;
use entity::product_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct ProductGateway {
    db: DbConn,
}

impl ProductGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }
}

#[async_trait]
impl Gateway<Product, product_entity::Model, product_entity::ActiveModel> for ProductGateway {
    async fn persist(&self, entity: Product) -> Result<product_entity::ActiveModel, DbErr> {
        let active_model = ProductEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            ProductQuery::delete_many().filter(product_entity::Column::Id.eq(id)),
            product_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<product_entity::Model>, DbErr> {
        tenant_select(ProductQuery::find(), product_entity::Column::TenantId)
            .filter(product_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<product_entity::Model>, DbErr> {
        tenant_select(ProductQuery::find(), product_entity::Column::TenantId)
            .filter(product_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<product_entity::Model>, DbErr> {
        tenant_select(ProductQuery::find(), product_entity::Column::TenantId)
            .order_by_asc(product_entity::Column::Name)
            .all(&self.db)
            .await
    }
}
