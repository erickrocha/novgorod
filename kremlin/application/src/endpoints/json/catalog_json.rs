use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryJson {
    pub id: Option<i64>,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i64>,
    pub active: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CatalogAttributeJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub name: String,
    pub display_type: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CatalogAttributeValueJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub attribute_id: i64,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductAttributeJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub attribute_id: i64,
    pub required: bool,
    pub sort_order: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkuJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub code: String,
    pub variant_key: String,
    pub price_cents: i32,
    pub compare_at_price_cents: Option<i32>,
    pub weight_g: Option<i32>,
    pub width_mm: Option<i32>,
    pub height_mm: Option<i32>,
    pub length_mm: Option<i32>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInputJson {
    pub tenant_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i64>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductInputJson {
    pub tenant_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub active: bool,
    pub ncm: String,
    pub cest: Option<String>,
    pub origem_mercadoria: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkuInputJson {
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub code: String,
    pub variant_key: String,
    pub price_cents: i32,
    pub compare_at_price_cents: Option<i32>,
    pub weight_g: Option<i32>,
    pub width_mm: Option<i32>,
    pub height_mm: Option<i32>,
    pub length_mm: Option<i32>,
    pub active: bool,
}
