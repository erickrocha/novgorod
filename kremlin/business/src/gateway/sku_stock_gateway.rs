use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::sku_stock::{SkuStock, SkuStockEntityMapper};
use entity::prelude::SkuStockEntity as SkuStockQuery;
use entity::sku_stock_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter};

pub struct SkuStockGateway {
    db: DbConn,
}

impl SkuStockGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_sku_id(
        &self,
        sku_id: i64,
    ) -> Result<Option<sku_stock_entity::Model>, DbErr> {
        tenant_select(
            SkuStockQuery::find(),
            sku_stock_entity::Column::TenantId,
        )
        .filter(sku_stock_entity::Column::SkuId.eq(sku_id))
        .one(&self.db)
        .await
    }
}

#[async_trait]
impl Gateway<SkuStock, sku_stock_entity::Model, sku_stock_entity::ActiveModel> for SkuStockGateway {
    async fn persist(
        &self,
        entity: SkuStock,
    ) -> Result<sku_stock_entity::ActiveModel, DbErr> {
        let active_model = SkuStockEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            SkuStockQuery::delete_many()
                .filter(sku_stock_entity::Column::Id.eq(id)),
            sku_stock_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<sku_stock_entity::Model>, DbErr> {
        tenant_select(
            SkuStockQuery::find(),
            sku_stock_entity::Column::TenantId,
        )
        .filter(sku_stock_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<sku_stock_entity::Model>, DbErr> {
        tenant_select(
            SkuStockQuery::find(),
            sku_stock_entity::Column::TenantId,
        )
        .filter(sku_stock_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<sku_stock_entity::Model>, DbErr> {
        tenant_select(
            SkuStockQuery::find(),
            sku_stock_entity::Column::TenantId,
        )
        .all(&self.db)
        .await
    }
}
