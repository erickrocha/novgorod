use crate::commons::pagination::PageQuery;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CouponJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub code: String,
    pub campaign_id: Option<i64>,
    pub coupon_type: String,
    pub value: i32,
    pub min_order_cents: Option<i32>,
    pub max_uses: Option<i32>,
    pub max_uses_per_customer: Option<i32>,
    pub starts_at: Option<chrono::NaiveDateTime>,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub active: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CouponInputJson {
    pub tenant_id: Option<i64>,
    pub code: String,
    pub campaign_id: Option<i64>,
    pub coupon_type: String,
    pub value: i32,
    pub min_order_cents: Option<i32>,
    pub max_uses: Option<i32>,
    pub max_uses_per_customer: Option<i32>,
    pub starts_at: Option<chrono::NaiveDateTime>,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CouponPageQuery {
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

impl CouponPageQuery {
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
pub struct CouponRedemptionJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub coupon_id: i64,
    pub order_id: i64,
    pub customer_id: i64,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CouponRedemptionInputJson {
    pub tenant_id: Option<i64>,
    pub coupon_id: i64,
    pub order_id: i64,
    pub customer_id: i64,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CouponRedemptionPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "coupon_id")]
    pub coupon_id: Option<i64>,
    #[serde(alias = "order_id")]
    pub order_id: Option<i64>,
    #[serde(alias = "customer_id")]
    pub customer_id: Option<i64>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl CouponRedemptionPageQuery {
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
