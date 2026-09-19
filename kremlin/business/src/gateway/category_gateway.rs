use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::category::{Category, CategoryEntityMapper};
use entity::category_entity;
use entity::prelude::CategoryEntity as CategoryQuery;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder,
};

pub struct CategoryGateway {
    db: DbConn,
}

impl CategoryGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }
}

#[async_trait]
impl Gateway<Category, category_entity::Model, category_entity::ActiveModel> for CategoryGateway {
    async fn persist(&self, entity: Category) -> Result<category_entity::ActiveModel, DbErr> {
        let active_model = CategoryEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            CategoryQuery::delete_many().filter(category_entity::Column::Id.eq(id)),
            category_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<category_entity::Model>, DbErr> {
        tenant_select(CategoryQuery::find(), category_entity::Column::TenantId)
            .filter(category_entity::Column::Id.eq(id))
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<category_entity::Model>, DbErr> {
        tenant_select(CategoryQuery::find(), category_entity::Column::TenantId)
            .filter(category_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<category_entity::Model>, DbErr> {
        tenant_select(CategoryQuery::find(), category_entity::Column::TenantId)
            .order_by_asc(category_entity::Column::Name)
            .all(&self.db)
            .await
    }
}
