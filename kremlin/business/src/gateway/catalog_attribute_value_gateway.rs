use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::catalog_attribute_value::{
    CatalogAttributeValue, CatalogAttributeValueEntityMapper,
};
use entity::catalog_attribute_value_entity;
use entity::prelude::CatalogAttributeValueEntity as CatalogAttributeValueQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct CatalogAttributeValueGateway {
    db: DbConn,
}

impl CatalogAttributeValueGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_attribute_id(
        &self,
        attribute_id: i64,
    ) -> Result<Vec<catalog_attribute_value_entity::Model>, DbErr> {
        tenant_select(
            CatalogAttributeValueQuery::find(),
            catalog_attribute_value_entity::Column::TenantId,
        )
        .filter(catalog_attribute_value_entity::Column::AttributeId.eq(attribute_id))
        .order_by_asc(catalog_attribute_value_entity::Column::Value)
        .all(&self.db)
        .await
    }
}

#[async_trait]
impl
    Gateway<
        CatalogAttributeValue,
        catalog_attribute_value_entity::Model,
        catalog_attribute_value_entity::ActiveModel,
    > for CatalogAttributeValueGateway
{
    async fn persist(
        &self,
        entity: CatalogAttributeValue,
    ) -> Result<catalog_attribute_value_entity::ActiveModel, DbErr> {
        let active_model = CatalogAttributeValueEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CatalogAttributeValueQuery::delete_many()
                .filter(catalog_attribute_value_entity::Column::Id.eq(id)),
            catalog_attribute_value_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<catalog_attribute_value_entity::Model>, DbErr> {
        tenant_select(
            CatalogAttributeValueQuery::find(),
            catalog_attribute_value_entity::Column::TenantId,
        )
        .filter(catalog_attribute_value_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<catalog_attribute_value_entity::Model>, DbErr> {
        tenant_select(
            CatalogAttributeValueQuery::find(),
            catalog_attribute_value_entity::Column::TenantId,
        )
        .filter(catalog_attribute_value_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<catalog_attribute_value_entity::Model>, DbErr> {
        tenant_select(
            CatalogAttributeValueQuery::find(),
            catalog_attribute_value_entity::Column::TenantId,
        )
        .order_by_asc(catalog_attribute_value_entity::Column::Value)
        .all(&self.db)
        .await
    }
}
