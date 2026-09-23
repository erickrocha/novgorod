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
