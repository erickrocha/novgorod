use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductImagePresignItemRequest {
    pub original_filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub sku_id: Option<i64>,
    pub alt_text: Option<String>,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductImagePresignBatchRequest {
    pub images: Vec<ProductImagePresignItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductImagePresignItemResponse {
    pub upload_url: String,
    pub object_key: String,
    pub cdn_url: Option<String>,
    pub image: ProductImageJson,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductImageJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub sku_id: Option<i64>,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub is_primary: bool,
    pub storage_provider: String,
    pub bucket: String,
    pub object_key: String,
    pub original_filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub storage_status: String,
    pub cdn_url: Option<String>,
}
