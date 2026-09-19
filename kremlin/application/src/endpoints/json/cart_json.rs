use crate::commons::pagination::PageQuery;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CartJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub status: String,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CartInputJson {
    pub tenant_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub status: Option<String>,
    pub expires_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CartPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    pub status: Option<String>,
    #[serde(alias = "customer_id")]
    pub customer_id: Option<i64>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl CartPageQuery {
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CartItemJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub cart_id: i64,
    pub sku_id: i64,
    pub quantity: i32,
    pub unit_price_cents: i32,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CartItemInputJson {
    pub tenant_id: Option<i64>,
    pub cart_id: i64,
    pub sku_id: i64,
    pub quantity: i32,
    pub unit_price_cents: i32,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CartItemPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "cart_id")]
    pub cart_id: Option<i64>,
    #[serde(alias = "sku_id")]
    pub sku_id: Option<i64>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl CartItemPageQuery {
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
