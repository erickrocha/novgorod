use crate::commons::entity_mapper::EntityMapper;
use entity::product_image_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const STORAGE_PROVIDER_S3: &str = "s3";
pub const STATUS_PENDING: &str = "pending";
pub const STATUS_AVAILABLE: &str = "available";
pub const STATUS_DELETE_PENDING: &str = "delete_pending";
pub const STATUS_DELETED: &str = "deleted";
pub const STATUS_FAILED: &str = "failed";

pub fn compute_storage_identity_hash(provider: &str, bucket: &str, object_key: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(provider.as_bytes());
    hasher.update(b":");
    hasher.update(bucket.as_bytes());
    hasher.update(b":");
    hasher.update(object_key.as_bytes());
    hasher.finalize().to_vec()
}

pub fn binary_to_uuid_string(bytes: &[u8]) -> String {
    if bytes.len() == 16 {
        match Uuid::from_slice(bytes) {
            Ok(u) => u.to_string(),
            Err(_) => Uuid::nil().to_string(),
        }
    } else {
        Uuid::nil().to_string()
    }
}

pub fn uuid_string_to_binary(uuid_str: &str) -> Vec<u8> {
    Uuid::parse_str(uuid_str)
        .unwrap_or_else(|_| Uuid::nil())
        .as_bytes()
        .to_vec()
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductImage {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub sku_id: Option<i64>,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub is_primary: bool,
    pub storage_provider: String,
    pub bucket: String,
    pub object_key: String,
    pub storage_identity_hash: Option<Vec<u8>>,
    pub object_version: Option<String>,
    pub etag: Option<String>,
    pub checksum_sha256: Option<Vec<u8>>,
    pub original_filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width_px: Option<i32>,
    pub height_px: Option<i32>,
    pub storage_status: String,
    pub cdn_url: Option<String>,
}

pub struct ProductImageEntityMapper {}

impl EntityMapper<ProductImage, Model, ActiveModel> for ProductImageEntityMapper {
    fn build_active_model(d: ProductImage) -> ActiveModel {
        let storage_hash = d.storage_identity_hash.unwrap_or_else(|| {
            compute_storage_identity_hash(&d.storage_provider, &d.bucket, &d.object_key)
        });

        ActiveModel {
            id: match d.id {
                Some(id) => Set(id),
                None => NotSet,
            },
            uuid: match d.uuid {
                Some(ref uuid_str) => Set(uuid_string_to_binary(uuid_str)),
                None => NotSet,
            },
            tenant_id: Set(d.tenant_id),
            product_id: Set(d.product_id),
            sku_id: match d.sku_id {
                Some(sid) => Set(Some(sid)),
                None => Set(None),
            },
            alt_text: match d.alt_text {
                Some(alt) => Set(Some(alt)),
                None => Set(None),
            },
            sort_order: Set(d.sort_order),
            is_primary: Set(d.is_primary),
            storage_provider: Set(d.storage_provider),
            bucket: Set(d.bucket),
            object_key: Set(d.object_key),
            storage_identity_hash: Set(storage_hash),
            object_version: match d.object_version {
                Some(ov) => Set(Some(ov)),
                None => Set(None),
            },
            etag: match d.etag {
                Some(etag) => Set(Some(etag)),
                None => Set(None),
            },
            checksum_sha256: match d.checksum_sha256 {
                Some(cs) => Set(Some(cs)),
                None => Set(None),
            },
            original_filename: Set(d.original_filename),
            mime_type: Set(d.mime_type),
            size_bytes: Set(d.size_bytes),
            width_px: match d.width_px {
                Some(w) => Set(Some(w)),
                None => Set(None),
            },
            height_px: match d.height_px {
                Some(h) => Set(Some(h)),
                None => Set(None),
            },
            storage_status: Set(d.storage_status),
            created_at: NotSet,
            created_by: NotSet,
            updated_at: NotSet,
            updated_by: NotSet,
            deleted_at: NotSet,
            deleted_by: NotSet,
        }
    }

    fn from_model(e: Model) -> ProductImage {
        ProductImage {
            id: Some(e.id),
            uuid: Some(binary_to_uuid_string(&e.uuid)),
            tenant_id: e.tenant_id,
            product_id: e.product_id,
            sku_id: e.sku_id,
            alt_text: e.alt_text,
            sort_order: e.sort_order,
            is_primary: e.is_primary,
            storage_provider: e.storage_provider,
            bucket: e.bucket,
            object_key: e.object_key,
            storage_identity_hash: Some(e.storage_identity_hash),
            object_version: e.object_version,
            etag: e.etag,
            checksum_sha256: e.checksum_sha256,
            original_filename: e.original_filename,
            mime_type: e.mime_type,
            size_bytes: e.size_bytes,
            width_px: e.width_px,
            height_px: e.height_px,
            storage_status: e.storage_status,
            cdn_url: None,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> ProductImage {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => ProductImage {
                id: e.id.take(),
                uuid: e.uuid.take().map(|bytes| binary_to_uuid_string(&bytes)),
                tenant_id: e.tenant_id.take().flatten(),
                product_id: e.product_id.take().unwrap_or_default(),
                sku_id: e.sku_id.take().flatten(),
                alt_text: e.alt_text.take().flatten(),
                sort_order: e.sort_order.take().unwrap_or_default(),
                is_primary: e.is_primary.take().unwrap_or_default(),
                storage_provider: e.storage_provider.take().unwrap_or_default(),
                bucket: e.bucket.take().unwrap_or_default(),
                object_key: e.object_key.take().unwrap_or_default(),
                storage_identity_hash: e.storage_identity_hash.take(),
                object_version: e.object_version.take().flatten(),
                etag: e.etag.take().flatten(),
                checksum_sha256: e.checksum_sha256.take().flatten(),
                original_filename: e.original_filename.take().unwrap_or_default(),
                mime_type: e.mime_type.take().unwrap_or_default(),
                size_bytes: e.size_bytes.take().unwrap_or_default(),
                width_px: e.width_px.take().flatten(),
                height_px: e.height_px.take().flatten(),
                storage_status: e.storage_status.take().unwrap_or_default(),
                cdn_url: None,
            },
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/domain/product_image.rs"]
mod tests;
