use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::sku_attribute_value::{SkuAttributeValue, SkuAttributeValueEntityMapper};
use entity::prelude::SkuAttributeValueEntity as SkuAttributeValueQuery;
use entity::sku_attribute_value_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter};

pub struct SkuAttributeValueGateway {
    db: DbConn,
}

pub type SkuAttributeGateway = SkuAttributeValueGateway;

impl SkuAttributeValueGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_sku_id(
        &self,
        sku_id: i64,
    ) -> Result<Vec<sku_attribute_value_entity::Model>, DbErr> {
        tenant_select(
            SkuAttributeValueQuery::find(),
            sku_attribute_value_entity::Column::TenantId,
        )
        .filter(sku_attribute_value_entity::Column::SkuId.eq(sku_id))
        .all(&self.db)
        .await
    }

    pub async fn find_by_product_id(
        &self,
        product_id: i64,
    ) -> Result<Vec<sku_attribute_value_entity::Model>, DbErr> {
        tenant_select(
            SkuAttributeValueQuery::find(),
            sku_attribute_value_entity::Column::TenantId,
        )
        .filter(sku_attribute_value_entity::Column::ProductId.eq(product_id))
        .all(&self.db)
        .await
    }

    pub async fn find_by_attribute_value_id(
        &self,
        attribute_value_id: i64,
    ) -> Result<Vec<sku_attribute_value_entity::Model>, DbErr> {
        tenant_select(
            SkuAttributeValueQuery::find(),
            sku_attribute_value_entity::Column::TenantId,
        )
        .filter(sku_attribute_value_entity::Column::AttributeValueId.eq(attribute_value_id))
        .all(&self.db)
        .await
    }
}

#[async_trait]
impl Gateway<
    SkuAttributeValue,
    sku_attribute_value_entity::Model,
    sku_attribute_value_entity::ActiveModel,
> for SkuAttributeValueGateway {
    async fn persist(
        &self,
        entity: SkuAttributeValue,
    ) -> Result<sku_attribute_value_entity::ActiveModel, DbErr> {
        let active_model = SkuAttributeValueEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            SkuAttributeValueQuery::delete_many()
                .filter(sku_attribute_value_entity::Column::Id.eq(id)),
            sku_attribute_value_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<sku_attribute_value_entity::Model>, DbErr> {
        tenant_select(
            SkuAttributeValueQuery::find(),
            sku_attribute_value_entity::Column::TenantId,
        )
        .filter(sku_attribute_value_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<sku_attribute_value_entity::Model>, DbErr> {
        tenant_select(
            SkuAttributeValueQuery::find(),
            sku_attribute_value_entity::Column::TenantId,
        )
        .filter(sku_attribute_value_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<sku_attribute_value_entity::Model>, DbErr> {
        tenant_select(
            SkuAttributeValueQuery::find(),
            sku_attribute_value_entity::Column::TenantId,
        )
        .all(&self.db)
        .await
    }
}
