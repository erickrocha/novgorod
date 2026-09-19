use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::product_attribute::{ProductAttribute, ProductAttributeEntityMapper};
use entity::prelude::ProductAttributeEntity as ProductAttributeQuery;
use entity::product_attribute_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct ProductAttributeGateway {
    db: DbConn,
}

impl ProductAttributeGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_product_id(
        &self,
        product_id: i64,
    ) -> Result<Vec<product_attribute_entity::Model>, DbErr> {
        tenant_select(
            ProductAttributeQuery::find(),
            product_attribute_entity::Column::TenantId,
        )
        .filter(product_attribute_entity::Column::ProductId.eq(product_id))
        .order_by_asc(product_attribute_entity::Column::SortOrder)
        .all(&self.db)
        .await
    }
}

#[async_trait]
impl Gateway<ProductAttribute, product_attribute_entity::Model, product_attribute_entity::ActiveModel>
    for ProductAttributeGateway
{
    async fn persist(
        &self,
        entity: ProductAttribute,
    ) -> Result<product_attribute_entity::ActiveModel, DbErr> {
        let active_model = ProductAttributeEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            ProductAttributeQuery::delete_many()
                .filter(product_attribute_entity::Column::Id.eq(id)),
            product_attribute_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<product_attribute_entity::Model>, DbErr> {
        tenant_select(
            ProductAttributeQuery::find(),
            product_attribute_entity::Column::TenantId,
        )
        .filter(product_attribute_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<product_attribute_entity::Model>, DbErr> {
        tenant_select(
            ProductAttributeQuery::find(),
            product_attribute_entity::Column::TenantId,
        )
        .filter(product_attribute_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<product_attribute_entity::Model>, DbErr> {
        tenant_select(
            ProductAttributeQuery::find(),
            product_attribute_entity::Column::TenantId,
        )
        .order_by_asc(product_attribute_entity::Column::SortOrder)
        .all(&self.db)
        .await
    }
}
