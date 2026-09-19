use crate::commons::pagination::PageQuery;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrdersJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub number: String,
    pub customer_id: i64,
    pub status: String,
    pub payment_status: String,
    pub subtotal_cents: i32,
    pub discount_cents: i32,
    pub shipping_cents: i32,
    pub tax_total_cents: i32,
    pub total_cents: i32,
    pub coupon_id: Option<i64>,
    pub coupon_code: Option<String>,
    pub ship_recipient: String,
    pub ship_cep: String,
    pub ship_logradouro: String,
    pub ship_numero: String,
    pub ship_complemento: Option<String>,
    pub ship_bairro: String,
    pub ship_cidade: String,
    pub ship_uf: String,
    pub placed_at: Option<chrono::NaiveDateTime>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrdersInputJson {
    pub tenant_id: Option<i64>,
    pub number: Option<String>,
    pub customer_id: i64,
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub subtotal_cents: i32,
    pub discount_cents: Option<i32>,
    pub shipping_cents: Option<i32>,
    pub tax_total_cents: Option<i32>,
    pub total_cents: i32,
    pub coupon_id: Option<i64>,
    pub coupon_code: Option<String>,
    pub ship_recipient: String,
    pub ship_cep: String,
    pub ship_logradouro: String,
    pub ship_numero: String,
    pub ship_complemento: Option<String>,
    pub ship_bairro: String,
    pub ship_cidade: String,
    pub ship_uf: String,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct OrdersPageQuery {
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

impl OrdersPageQuery {
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
pub struct OrderItemJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub order_id: i64,
    pub sku_id: i64,
    pub sku_code: String,
    pub product_name: String,
    pub attributes_desc: Option<String>,
    pub quantity: i32,
    pub unit_price_cents: i32,
    pub discount_cents: i32,
    pub ncm: String,
    pub cfop: String,
    pub csosn: String,
    pub icms_rate_bp: i32,
    pub tax_cents: i32,
    pub total_cents: i32,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemInputJson {
    pub tenant_id: Option<i64>,
    pub order_id: i64,
    pub sku_id: i64,
    pub sku_code: String,
    pub product_name: String,
    pub attributes_desc: Option<String>,
    pub quantity: i32,
    pub unit_price_cents: i32,
    pub discount_cents: Option<i32>,
    pub ncm: String,
    pub cfop: String,
    pub csosn: String,
    pub icms_rate_bp: i32,
    pub tax_cents: i32,
    pub total_cents: i32,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "order_id")]
    pub order_id: Option<i64>,
    #[serde(alias = "sku_id")]
    pub sku_id: Option<i64>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl OrderItemPageQuery {
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
pub struct OrderStatusHistoryJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub order_id: i64,
    pub from_status: Option<String>,
    pub to_status: String,
    pub actor_type: String,
    pub actor_id: Option<i64>,
    pub note: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatusHistoryInputJson {
    pub tenant_id: Option<i64>,
    pub order_id: i64,
    pub from_status: Option<String>,
    pub to_status: String,
    pub actor_type: String,
    pub actor_id: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatusHistoryPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "order_id")]
    pub order_id: Option<i64>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl OrderStatusHistoryPageQuery {
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
