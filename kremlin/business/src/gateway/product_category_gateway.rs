use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::string_to_uuid;
use crate::commons::gateway::{Gateway, tenant_delete, tenant_select};
use crate::domain::product_category::{ProductCategory, ProductCategoryEntityMapper};
use entity::prelude::ProductCategoryEntity as ProductCategoryQuery;
use entity::product_category_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter};

pub struct ProductCategoryGateway {
    db: DbConn,
}

impl ProductCategoryGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbConn {
        &self.db
    }

    pub async fn find_by_product_id(
        &self,
        product_id: i64,
    ) -> Result<Vec<product_category_entity::Model>, DbErr> {
        tenant_select(
            ProductCategoryQuery::find(),
            product_category_entity::Column::TenantId,
        )
        .filter(product_category_entity::Column::ProductId.eq(product_id))
        .all(&self.db)
        .await
    }

    pub async fn find_by_category_id(
        &self,
        category_id: i64,
    ) -> Result<Vec<product_category_entity::Model>, DbErr> {
        tenant_select(
            ProductCategoryQuery::find(),
            product_category_entity::Column::TenantId,
        )
        .filter(product_category_entity::Column::CategoryId.eq(category_id))
        .all(&self.db)
        .await
    }

    pub async fn find_primary_for_product(
        &self,
        product_id: i64,
    ) -> Result<Option<product_category_entity::Model>, DbErr> {
        tenant_select(
            ProductCategoryQuery::find(),
            product_category_entity::Column::TenantId,
        )
        .filter(product_category_entity::Column::ProductId.eq(product_id))
        .filter(product_category_entity::Column::IsPrimary.eq(true))
        .one(&self.db)
        .await
    }
}

#[async_trait]
impl Gateway<ProductCategory, product_category_entity::Model, product_category_entity::ActiveModel>
    for ProductCategoryGateway
{
    async fn persist(
        &self,
        entity: ProductCategory,
    ) -> Result<product_category_entity::ActiveModel, DbErr> {
        let active_model = ProductCategoryEntityMapper::build_active_model(entity);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        tenant_delete(
            ProductCategoryQuery::delete_many()
                .filter(product_category_entity::Column::Id.eq(id)),
            product_category_entity::Column::TenantId,
        )
        .exec(&self.db)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<product_category_entity::Model>, DbErr> {
        tenant_select(
            ProductCategoryQuery::find(),
            product_category_entity::Column::TenantId,
        )
        .filter(product_category_entity::Column::Id.eq(id))
        .one(&self.db)
        .await
    }

    async fn find_by_uuid(
        &self,
        uuid: String,
    ) -> Result<Option<product_category_entity::Model>, DbErr> {
        tenant_select(
            ProductCategoryQuery::find(),
            product_category_entity::Column::TenantId,
        )
        .filter(product_category_entity::Column::Uuid.eq(string_to_uuid(&uuid)))
        .one(&self.db)
        .await
    }

    async fn find_all(&self) -> Result<Vec<product_category_entity::Model>, DbErr> {
        tenant_select(
            ProductCategoryQuery::find(),
            product_category_entity::Column::TenantId,
        )
        .all(&self.db)
        .await
    }
}
