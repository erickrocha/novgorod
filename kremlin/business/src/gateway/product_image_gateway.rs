use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::{tenant_select, Gateway};
use crate::domain::product_image::{
    uuid_string_to_binary, ProductImage, ProductImageEntityMapper, STATUS_AVAILABLE, STATUS_DELETED,
};
use entity::prelude::ProductImageEntity as ProductImageQuery;
use entity::product_image_entity;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder, Set,
};

pub struct ProductImageGateway {
    db: DbConn,
}

impl ProductImageGateway {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub async fn find_by_product_id(
        &self,
        product_id: i64,
    ) -> Result<Vec<product_image_entity::Model>, DbErr> {
        let query = ProductImageQuery::find()
            .filter(product_image_entity::Column::ProductId.eq(product_id))
            .filter(product_image_entity::Column::DeletedAt.is_null());

        tenant_select(query, product_image_entity::Column::TenantId)
            .order_by_asc(product_image_entity::Column::SortOrder)
            .order_by_asc(product_image_entity::Column::Id)
            .all(&self.db)
            .await
    }

    pub async fn find_by_bucket_and_key(
        &self,
        bucket: &str,
        object_key: &str,
    ) -> Result<Option<product_image_entity::Model>, DbErr> {
        ProductImageQuery::find()
            .filter(product_image_entity::Column::Bucket.eq(bucket))
            .filter(product_image_entity::Column::ObjectKey.eq(object_key))
            .filter(product_image_entity::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
    }

    pub async fn mark_as_available(
        &self,
        id: i64,
        etag: Option<String>,
        size_bytes: Option<i64>,
    ) -> Result<Option<product_image_entity::Model>, DbErr> {
        if let Some(existing) = ProductImageQuery::find_by_id(id).one(&self.db).await? {
            let mut active: product_image_entity::ActiveModel = existing.into();
            active.storage_status = Set(STATUS_AVAILABLE.to_string());
            if let Some(e) = etag {
                active.etag = Set(Some(e));
            }
            if let Some(s) = size_bytes {
                active.size_bytes = Set(s);
            }
            let updated = active.update(&self.db).await?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    pub async fn clear_primary_for_product(&self, product_id: i64) -> Result<(), DbErr> {
        let images = ProductImageQuery::find()
            .filter(product_image_entity::Column::ProductId.eq(product_id))
            .filter(product_image_entity::Column::IsPrimary.eq(true))
            .filter(product_image_entity::Column::DeletedAt.is_null())
            .all(&self.db)
            .await?;

        for img in images {
            let mut active: product_image_entity::ActiveModel = img.into();
            active.is_primary = Set(false);
            active.update(&self.db).await?;
        }

        Ok(())
    }

    pub async fn set_primary(&self, id: i64, product_id: i64) -> Result<(), DbErr> {
        self.clear_primary_for_product(product_id).await?;

        if let Some(img) = ProductImageQuery::find_by_id(id).one(&self.db).await? {
            if img.product_id == product_id {
                let mut active: product_image_entity::ActiveModel = img.into();
                active.is_primary = Set(true);
                active.update(&self.db).await?;
            }
        }

        Ok(())
    }

    pub async fn soft_delete(&self, id: i64) -> Result<Option<product_image_entity::Model>, DbErr> {
        if let Some(existing) = ProductImageQuery::find_by_id(id).one(&self.db).await? {
            let mut active: product_image_entity::ActiveModel = existing.into();
            active.deleted_at = Set(Some(chrono::Utc::now().naive_utc()));
            active.storage_status = Set(STATUS_DELETED.to_string());
            let updated = active.update(&self.db).await?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl Gateway<ProductImage, product_image_entity::Model, product_image_entity::ActiveModel>
    for ProductImageGateway
{
    async fn persist(&self, domain: ProductImage) -> Result<product_image_entity::ActiveModel, DbErr> {
        let active_model = ProductImageEntityMapper::build_active_model(domain);
        active_model.save(&self.db).await
    }

    async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        ProductImageQuery::delete_by_id(id).exec(&self.db).await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<product_image_entity::Model>, DbErr> {
        let query = ProductImageQuery::find()
            .filter(product_image_entity::Column::Id.eq(id))
            .filter(product_image_entity::Column::DeletedAt.is_null());

        tenant_select(query, product_image_entity::Column::TenantId)
            .one(&self.db)
            .await
    }

    async fn find_by_uuid(&self, uuid: String) -> Result<Option<product_image_entity::Model>, DbErr> {
        let bytes = uuid_string_to_binary(&uuid);
        let query = ProductImageQuery::find()
            .filter(product_image_entity::Column::Uuid.eq(bytes))
            .filter(product_image_entity::Column::DeletedAt.is_null());

        tenant_select(query, product_image_entity::Column::TenantId)
            .one(&self.db)
            .await
    }

    async fn find_all(&self) -> Result<Vec<product_image_entity::Model>, DbErr> {
        let query = ProductImageQuery::find()
            .filter(product_image_entity::Column::DeletedAt.is_null());

        tenant_select(query, product_image_entity::Column::TenantId)
            .order_by_asc(product_image_entity::Column::SortOrder)
            .all(&self.db)
            .await
    }
}
