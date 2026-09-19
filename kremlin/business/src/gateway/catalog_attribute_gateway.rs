use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::catalog_attribute::{CatalogAttribute, CatalogAttributeEntityMapper};
use entity::catalog_attribute_entity;
use entity::prelude::CatalogAttributeEntity as CatalogAttributeQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct CatalogAttributeGateway {
    db: DbConn,
}

impl CatalogAttributeGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }
}

#[async_trait]
impl Gateway<CatalogAttribute, catalog_attribute_entity::Model, catalog_attribute_entity::ActiveModel>
    for CatalogAttributeGateway
{
    async fn persist(
        &self,
        entity: CatalogAttribute,
    ) -> Result<catalog_attribute_entity::ActiveModel, DbErr> {
        let active_model = CatalogAttributeEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CatalogAttributeQuery::delete_many()
                .filter(catalog_attribute_entity::Column::Id.eq(id)),
            catalog_attribute_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<catalog_attribute_entity::Model>, DbErr> {
        tenant_select(
            CatalogAttributeQuery::find(),
            catalog_attribute_entity::Column::TenantId,
        )
        .filter(catalog_attribute_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<catalog_attribute_entity::Model>, DbErr> {
        tenant_select(
            CatalogAttributeQuery::find(),
            catalog_attribute_entity::Column::TenantId,
        )
        .filter(catalog_attribute_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<catalog_attribute_entity::Model>, DbErr> {
        tenant_select(
            CatalogAttributeQuery::find(),
            catalog_attribute_entity::Column::TenantId,
        )
        .order_by_asc(catalog_attribute_entity::Column::Name)
        .all(&self.db)
        .await
    }
}
