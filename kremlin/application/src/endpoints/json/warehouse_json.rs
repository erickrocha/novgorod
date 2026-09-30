use crate::commons::pagination::PageQuery;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub name: String,
    pub origin_cep: String,
    pub street: Option<String>,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub district: Option<String>,
    pub city: String,
    pub uf: String,
    pub is_default: bool,
    pub active: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseInputJson {
    pub tenant_id: Option<i64>,
    pub name: String,
    pub origin_cep: String,
    pub street: Option<String>,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub district: Option<String>,
    pub city: String,
    pub uf: String,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default = "default_true")]
    pub active: bool,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct WarehousePageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    pub uf: Option<String>,
}

impl WarehousePageQuery {
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
