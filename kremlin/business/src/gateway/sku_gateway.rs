use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::sku::{Sku, SkuEntityMapper};
use entity::prelude::SkuEntity as SkuQuery;
use entity::sku_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct SkuGateway {
    db: DbConn,
}

impl SkuGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_product_id(
        &self,
        product_id: i64,
    ) -> Result<Vec<sku_entity::Model>, DbErr> {
        tenant_select(SkuQuery::find(), sku_entity::Column::TenantId)
            .filter(sku_entity::Column::ProductId.eq(product_id))
            .order_by_asc(sku_entity::Column::Code)
            .all(&self.db)
            .await
    }

    pub async fn find_by_code(&self, code: String) -> Result<Option<sku_entity::Model>, DbErr> {
        tenant_select(SkuQuery::find(), sku_entity::Column::TenantId)
            .filter(sku_entity::Column::Code.eq(code))
            .one(&self.db)
            .await
    }
}

#[async_trait]
impl Gateway<Sku, sku_entity::Model, sku_entity::ActiveModel> for SkuGateway {
    async fn persist(&self, entity: Sku) -> Result<sku_entity::ActiveModel, DbErr> {
        let active_model = SkuEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            SkuQuery::delete_many().filter(sku_entity::Column::Id.eq(id)),
            sku_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<sku_entity::Model>, DbErr> {
        tenant_select(SkuQuery::find(), sku_entity::Column::TenantId)
            .filter(sku_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<sku_entity::Model>, DbErr> {
        tenant_select(SkuQuery::find(), sku_entity::Column::TenantId)
            .filter(sku_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<sku_entity::Model>, DbErr> {
        tenant_select(SkuQuery::find(), sku_entity::Column::TenantId)
            .order_by_asc(sku_entity::Column::Code)
            .all(&self.db)
            .await
    }
}
