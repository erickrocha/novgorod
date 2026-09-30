use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::warehouse::{Warehouse, WarehouseEntityMapper};
use entity::prelude::WarehouseEntity as WarehouseQuery;
use entity::warehouse_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct WarehouseGateway {
    db: DbConn,
}

impl WarehouseGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }
}

#[async_trait]
impl Gateway<Warehouse, warehouse_entity::Model, warehouse_entity::ActiveModel>
    for WarehouseGateway
{
    async fn persist(
        &self,
        entity: Warehouse,
    ) -> Result<warehouse_entity::ActiveModel, DbErr> {
        let active_model = WarehouseEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            WarehouseQuery::delete_many().filter(warehouse_entity::Column::Id.eq(id)),
            warehouse_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<warehouse_entity::Model>, DbErr> {
        tenant_select(
            WarehouseQuery::find(),
            warehouse_entity::Column::TenantId,
        )
        .filter(warehouse_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<warehouse_entity::Model>, DbErr> {
        tenant_select(
            WarehouseQuery::find(),
            warehouse_entity::Column::TenantId,
        )
        .filter(warehouse_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<warehouse_entity::Model>, DbErr> {
        tenant_select(
            WarehouseQuery::find(),
            warehouse_entity::Column::TenantId,
        )
        .order_by_asc(warehouse_entity::Column::Name)
        .all(&self.db)
        .await
    }
}
