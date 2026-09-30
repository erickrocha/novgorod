use crate::commons::pagination::PageQuery;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

fn default_transit_days_min() -> i32 {
    1
}
fn default_transit_days_max() -> i32 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShippingRateJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub origin_warehouse_id: Option<i64>,
    pub region_name: Option<String>,
    pub uf: String,
    pub destination_cep_start: Option<String>,
    pub destination_cep_end: Option<String>,
    pub price_cents: i32,
    pub transit_days_min: i32,
    pub transit_days_max: i32,
    pub max_weight_g: Option<i32>,
    pub extra_weight_per_kg_cents: Option<i32>,
    pub free_shipping_threshold_cents: Option<i32>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShippingRateInputJson {
    pub tenant_id: Option<i64>,
    pub origin_warehouse_id: Option<i64>,
    pub region_name: Option<String>,
    pub uf: String,
    pub destination_cep_start: Option<String>,
    pub destination_cep_end: Option<String>,
    pub price_cents: i32,
    #[serde(default = "default_transit_days_min")]
    pub transit_days_min: i32,
    #[serde(default = "default_transit_days_max")]
    pub transit_days_max: i32,
    pub max_weight_g: Option<i32>,
    pub extra_weight_per_kg_cents: Option<i32>,
    pub free_shipping_threshold_cents: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ShippingRatePageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    pub uf: Option<String>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
    #[serde(alias = "origin_warehouse_id")]
    pub origin_warehouse_id: Option<i64>,
}

impl ShippingRatePageQuery {
    pub fn to_page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }
}
