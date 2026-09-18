use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::product_image::{
    compute_storage_identity_hash, ProductImage, ProductImageEntityMapper, STATUS_PENDING,
    STORAGE_PROVIDER_S3,
};
use crate::gateway::product_image_gateway::ProductImageGateway;
use crate::gateway::storage_gateway::StorageGateway;
use entity::product_image_entity::Model;
use sea_orm::TryIntoModel;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateImageUploadRequest {
    pub original_filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub sku_id: Option<i64>,
    pub alt_text: Option<String>,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUploadResult {
    pub image: ProductImage,
    pub upload_url: String,
}

pub struct ProductImageUseCase {
    gateway: ProductImageGateway,
    storage: StorageGateway,
}

impl ProductImageUseCase {
    pub fn new(gateway: ProductImageGateway, storage: StorageGateway) -> Self {
        Self { gateway, storage }
    }

    fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
            .collect()
    }

    pub async fn request_batch_presigned_upload(
        &self,
        product_id: i64,
        tenant_id: Option<i64>,
        requests: Vec<CreateImageUploadRequest>,
    ) -> Result<Vec<ImageUploadResult>, BusinessError> {
        let mut results = Vec::new();
        let existing_images = self
            .gateway
            .find_by_product_id(product_id)
            .await
            .map_err(|e| BusinessError::new(format!("Failed to query existing product images: {}", e)))?;

        let has_existing_images = !existing_images.is_empty();
        let mut has_primary_set = existing_images.iter().any(|img| img.is_primary);

        for (index, req) in requests.into_iter().enumerate() {
            let clean_filename = Self::sanitize_filename(&req.original_filename);
            let unique_id = Uuid::new_v4().to_string();
            let object_key = format!(
                "tenants/{}/products/{}/images/{}-{}",
                tenant_id.unwrap_or(0),
                product_id,
                unique_id,
                clean_filename
            );

            let is_primary = if req.is_primary.unwrap_or(false) {
                if !has_primary_set {
                    has_primary_set = true;
                    true
                } else {
                    self.gateway
                        .clear_primary_for_product(product_id)
                        .await
                        .map_err(|e| BusinessError::new(format!("Failed to clear primary flag: {}", e)))?;
                    true
                }
            } else if !has_existing_images && index == 0 && !has_primary_set {
                has_primary_set = true;
                true
            } else {
                false
            };

            let sort_order = req
                .sort_order
                .unwrap_or_else(|| (existing_images.len() + index) as i32);

            let bucket = self.storage.bucket_name().to_string();
            let storage_hash = compute_storage_identity_hash(STORAGE_PROVIDER_S3, &bucket, &object_key);

            let domain = ProductImage {
                id: None,
                uuid: Some(unique_id),
                tenant_id,
                product_id,
                sku_id: req.sku_id,
                alt_text: req.alt_text,
                sort_order,
                is_primary,
                storage_provider: STORAGE_PROVIDER_S3.to_string(),
                bucket: bucket.clone(),
                object_key: object_key.clone(),
                storage_identity_hash: Some(storage_hash),
                object_version: None,
                etag: None,
                checksum_sha256: None,
                original_filename: req.original_filename,
                mime_type: req.mime_type.clone(),
                size_bytes: req.size_bytes,
                width_px: None,
                height_px: None,
                storage_status: STATUS_PENDING.to_string(),
                cdn_url: Some(self.storage.get_cdn_url(&object_key)),
            };

            let saved_active = self
                .gateway
                .persist(domain.clone())
                .await
                .map_err(|e| BusinessError::new(format!("Failed to persist product image record: {}", e)))?;

            let saved_model: Result<Model, _> = saved_active.try_into_model();
            let mut persisted_domain = match saved_model {
                Ok(m) => ProductImageEntityMapper::from_model(m),
                Err(_) => domain,
            };
            persisted_domain.cdn_url = Some(self.storage.get_cdn_url(&object_key));

            let upload_url = self
                .storage
                .generate_presigned_upload_url(&object_key, &req.mime_type, 900)
                .await
                .map_err(|e| BusinessError::new(e))?;

            results.push(ImageUploadResult {
                image: persisted_domain,
                upload_url,
            });
        }

        Ok(results)
    }

    pub async fn list_by_product(&self, product_id: i64) -> Result<Vec<ProductImage>, BusinessError> {
        let models = self
            .gateway
            .find_by_product_id(product_id)
            .await
            .map_err(|e| BusinessError::new(format!("Failed to list images: {}", e)))?;

        let mut list = Vec::new();
        for m in models {
            let mut domain = ProductImageEntityMapper::from_model(m);
            domain.cdn_url = Some(self.storage.get_cdn_url(&domain.object_key));
            list.push(domain);
        }

        Ok(list)
    }

    pub async fn delete_image(&self, product_id: i64, image_id: i64) -> Result<(), BusinessError> {
        let existing = self
            .gateway
            .find_by_id(image_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?
            .ok_or_else(|| BusinessError::new("Product image not found".to_string()))?;

        if existing.product_id != product_id {
            return Err(BusinessError::new("Image does not belong to product".to_string()));
        }

        self.gateway
            .soft_delete(image_id)
            .await
            .map_err(|e| BusinessError::new(format!("Failed to delete product image: {}", e)))?;

        let _ = self.storage.delete_object(&existing.object_key).await;

        Ok(())
    }

    pub async fn set_primary(&self, product_id: i64, image_id: i64) -> Result<(), BusinessError> {
        let existing = self
            .gateway
            .find_by_id(image_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?
            .ok_or_else(|| BusinessError::new("Product image not found".to_string()))?;

        if existing.product_id != product_id {
            return Err(BusinessError::new("Image does not belong to product".to_string()));
        }

        self.gateway
            .set_primary(image_id, product_id)
            .await
            .map_err(|e| BusinessError::new(format!("Failed to set primary image: {}", e)))?;

        Ok(())
    }

    pub async fn handle_s3_upload_notification(
        &self,
        bucket: &str,
        object_key: &str,
        etag: Option<String>,
        size_bytes: Option<i64>,
    ) -> Result<bool, BusinessError> {
        let clean_key = object_key.replace("%2F", "/");

        if let Some(existing) = self
            .gateway
            .find_by_bucket_and_key(bucket, &clean_key)
            .await
            .map_err(|e| BusinessError::new(format!("Database lookup error: {}", e)))?
        {
            self.gateway
                .mark_as_available(existing.id, etag, size_bytes)
                .await
                .map_err(|e| BusinessError::new(format!("Failed to update image status: {}", e)))?;
            log::info!(
                "Product image {} (id={}) marked as available via S3 notification",
                clean_key,
                existing.id
            );
            Ok(true)
        } else {
            log::warn!(
                "Received S3 upload notification for unmapped image: bucket={}, key={}",
                bucket,
                clean_key
            );
            Ok(false)
        }
    }
}
