use crate::commons::pagination::PageQuery;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductCategoryJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub category_id: i64,
    pub is_primary: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductCategoryInputJson {
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub category_id: i64,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ProductCategoryPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "product_id")]
    pub product_id: Option<i64>,
    #[serde(alias = "category_id")]
    pub category_id: Option<i64>,
    #[serde(alias = "is_primary")]
    pub is_primary: Option<bool>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl ProductCategoryPageQuery {
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
