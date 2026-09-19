use crate::commons::pagination::PageQuery;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CampaignJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub name: String,
    pub campaign_type: String,
    pub value: i32,
    pub scope: String,
    pub starts_at: chrono::NaiveDateTime,
    pub ends_at: chrono::NaiveDateTime,
    pub active: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CampaignInputJson {
    pub tenant_id: Option<i64>,
    pub name: String,
    pub campaign_type: String,
    pub value: i32,
    pub scope: String,
    pub starts_at: chrono::NaiveDateTime,
    pub ends_at: chrono::NaiveDateTime,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CampaignPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    pub active: Option<bool>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl CampaignPageQuery {
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
pub struct CampaignTargetJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub campaign_id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTargetInputJson {
    pub tenant_id: Option<i64>,
    pub campaign_id: i64,
    pub target_type: String,
    pub target_id: i64,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTargetPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "campaign_id")]
    pub campaign_id: Option<i64>,
    #[serde(alias = "target_type")]
    pub target_type: Option<String>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl CampaignTargetPageQuery {
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
