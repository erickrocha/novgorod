use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WebStoreProductJson {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub price_cents: Option<i32>,
    pub compare_at_price_cents: Option<i32>,
    pub primary_image_url: Option<String>,
    pub primary_image_alt: Option<String>,
    pub category_slugs: Vec<String>,
    pub rating: Option<f32>,
    pub review_count: Option<i32>,
    pub is_featured: bool,
    pub is_new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WebStoreSellerJson {
    pub id: i64,
    pub business_name: String,
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub web_site: Option<String>,
    pub locality: Option<String>,
    pub administrative_area: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WebStoreImageJson {
    pub id: i64,
    pub url: String,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub is_primary: bool,
    pub width_px: Option<i32>,
    pub height_px: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WebStoreSkuAttributeValueJson {
    pub attribute_id: i64,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WebStoreSkuJson {
    pub id: i64,
    pub uuid: String,
    pub code: String,
    pub variant_key: String,
    pub price_cents: i32,
    pub compare_at_price_cents: Option<i32>,
    pub weight_g: Option<i32>,
    pub width_mm: Option<i32>,
    pub height_mm: Option<i32>,
    pub length_mm: Option<i32>,
    pub active: bool,
    pub stock: i32,
    pub attributes: Vec<WebStoreSkuAttributeValueJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WebStoreAttributeJson {
    pub id: i64,
    pub attribute_id: i64,
    pub name: String,
    pub display_type: String,
    pub required: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WebStoreProductDetailJson {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub active: bool,
    pub ncm: String,
    pub cest: Option<String>,
    pub origem_mercadoria: i16,
    pub seller: WebStoreSellerJson,
    pub images: Vec<WebStoreImageJson>,
    pub skus: Vec<WebStoreSkuJson>,
    pub attributes: Vec<WebStoreAttributeJson>,
    pub category_slugs: Vec<String>,
    pub rating: Option<f32>,
    pub review_count: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct WebStorePageQuery {
    pub cursor: Option<i64>,
    pub limit: Option<u64>,
    pub q: Option<String>,
    pub category: Option<String>,
    pub min_price: Option<i32>,
    pub max_price: Option<i32>,
    pub sort_by: Option<String>,
}
