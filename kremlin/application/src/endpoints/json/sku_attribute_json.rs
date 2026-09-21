use crate::commons::pagination::PageQuery;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkuAttributeValueJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub sku_id: i64,
    pub product_attribute_id: i64,
    pub attribute_id: i64,
    pub attribute_value_id: i64,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub updated_by: Option<String>,
}

#[allow(dead_code)]
pub type SkuAttributeJson = SkuAttributeValueJson;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkuAttributeValueInputJson {
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub sku_id: i64,
    pub product_attribute_id: i64,
    pub attribute_id: i64,
    pub attribute_value_id: i64,
}

#[allow(dead_code)]
pub type SkuAttributeInputJson = SkuAttributeValueInputJson;

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct SkuAttributeValuePageQuery {
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
    #[serde(alias = "sku_id")]
    pub sku_id: Option<i64>,
    #[serde(alias = "attribute_id")]
    pub attribute_id: Option<i64>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

#[allow(dead_code)]
pub type SkuAttributePageQuery = SkuAttributeValuePageQuery;

impl SkuAttributeValuePageQuery {
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
