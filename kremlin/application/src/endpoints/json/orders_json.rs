use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};
use crate::commons::pagination::PageQuery;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseJson {
    pub id: i64,
    pub uuid: String,
    pub customer_id: i64,
    pub customer_name: String,
    pub customer_tax_id: String,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub currency: String,
    pub status: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub shipping_cents: i64,
    pub tax_total_cents: i64,
    pub total_cents: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrdersJson {
    pub id: i64,
    pub uuid: String,
    pub purchase_id: i64,
    pub tenant_id: i64,
    pub number: String,
    pub customer_id: i64,
    pub customer_name: String,
    pub customer_tax_id: String,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub status: String,
    pub payment_status: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub shipping_cents: i64,
    pub tax_total_cents: i64,
    pub total_cents: i64,
    pub coupon_id: Option<i64>,
    pub coupon_code: Option<String>,
    pub placed_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: i64,
    pub order_id: i64,
    pub sku_id: i64,
    pub sku_code: String,
    pub product_name: String,
    pub attributes_desc: Option<String>,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub discount_cents: i64,
    pub ncm: String,
    pub cfop: Option<String>,
    pub csosn: Option<String>,
    pub icms_rate_bp: i32,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatusHistoryJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: i64,
    pub order_id: i64,
    pub from_status: Option<String>,
    pub to_status: String,
    pub actor_type: String,
    pub actor_id: Option<i64>,
    pub note: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderAddressJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: i64,
    pub order_id: i64,
    pub address_type: String,
    pub recipient: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub locality: String,
    pub administrative_area: String,
    pub postal_code: String,
    pub country_code: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentJson {
    pub id: i64,
    pub uuid: String,
    pub purchase_id: i64,
    pub method: String,
    pub status: String,
    pub installments: i32,
    pub amount_cents: i64,
    pub currency: String,
    pub gateway_provider: Option<String>,
    pub gateway_reference: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreditCardDetailsJson {
    pub id: i64,
    pub uuid: String,
    pub payment_id: i64,
    pub cardholder_name: String,
    pub brand: String,
    pub last_four_digits: String,
    pub expiration_month: i32,
    pub expiration_year: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentTransactionJson {
    pub id: i64,
    pub uuid: String,
    pub payment_id: i64,
    pub operation: String,
    pub status: String,
    pub amount_cents: i64,
    pub currency: String,
    pub gateway_provider: Option<String>,
    pub gateway_transaction_id: Option<String>,
    pub response_code: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentAllocationJson {
    pub id: i64,
    pub uuid: String,
    pub purchase_id: i64,
    pub payment_id: i64,
    pub order_id: i64,
    pub amount_cents: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}


#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailJson {
 #[serde(flatten)]
 pub order: OrdersJson,
 pub items: Vec<OrderItemJson>,
 pub addresses: Vec<OrderAddressJson>,
 pub allocations: Vec<PaymentAllocationJson>,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentDetailJson {
 #[serde(flatten)]
 pub payment: PaymentJson,
 pub card: Option<CreditCardDetailsJson>,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseDetailJson {
 #[serde(flatten)]
 pub purchase: PurchaseJson,
 pub orders: Vec<OrderDetailJson>,
 pub payments: Vec<PaymentDetailJson>,
}
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AddressInputJson {
 pub recipient: String,
 pub address_line1: String,
 pub address_line2: Option<String>,
 pub locality: String,
 pub administrative_area: String,
 pub postal_code: String,
 pub country_code: String,
}
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseItemInputJson {
 pub sku_id: i64,
 pub quantity: i32,
}
/// Prices come from the catalog. No inventory is reserved or card charged.
/// Shipping, discounts and taxes are zero in this persistence phase.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatePurchaseInputJson {
 pub items: Vec<PurchaseItemInputJson>,
 pub shipping_address: AddressInputJson,
 pub billing_address: Option<AddressInputJson>,
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
 #[serde(alias = "tenant_id")]
 pub tenant_id: Option<i64>,
 #[serde(alias = "customer_id")]
 pub customer_id: Option<i64>,
}
impl OrdersPageQuery {
 pub fn to_page_query(&self) -> PageQuery {
 PageQuery { page: self.page, page_size: self.page_size, q: self.q.clone(), sort_by: self.sort_by.clone(), sort_dir: self.sort_dir.clone() }
 }
}
