use crate::commons::pagination::PageQuery;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaxRuleJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub uf_origem: String,
    pub uf_destino: String,
    pub ncm_prefix: Option<String>,
    pub regime: String,
    pub csosn: Option<String>,
    pub cfop: String,
    pub icms_rate_bp: i32,
    pub active: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaxRuleInputJson {
    pub tenant_id: Option<i64>,
    pub uf_origem: String,
    pub uf_destino: String,
    pub ncm_prefix: Option<String>,
    pub regime: String,
    pub csosn: Option<String>,
    pub cfop: String,
    pub icms_rate_bp: i32,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TaxRulePageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    pub uf_origem: Option<String>,
    pub uf_destino: Option<String>,
    pub active: Option<bool>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl TaxRulePageQuery {
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
