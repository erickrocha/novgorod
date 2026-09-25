use crate::{domain::marketplace::{AddressInput, CreatePurchaseInput, PurchaseError, PurchaseItemInput}, gateway::purchase_gateway::PurchaseGateway};
use chrono::{Duration, NaiveDateTime, Utc};
use entity::{checkout_quote_entity, coupon_entity, coupon_redemption_entity, customer_address_entity, shipping_rate_entity};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DbBackend, DbConn, EntityTrait, PaginatorTrait, QueryFilter, Set, Statement};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SellerCoupon { pub tenant_id: i64, pub code: String }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuoteRequest {
    pub items: Vec<PurchaseItemInput>,
    pub address_id: Option<i64>,
    pub shipping_address: Option<AddressInput>,
    #[serde(default)] pub coupons: Vec<SellerCoupon>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuoteItem {
    pub sku_id: i64, pub tenant_id: i64, pub name: String,
    pub quantity: i32, pub unit_price_cents: i64, pub total_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SellerQuote {
    pub tenant_id: i64, pub subtotal_cents: i64, pub discount_cents: i64,
    pub shipping_cents: i64, pub total_cents: i64,
    pub coupon_id: Option<i64>, pub coupon_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResult {
    pub id: i64, pub expires_at: NaiveDateTime,
    pub shipping_address: AddressInput,
    pub items: Vec<QuoteItem>, pub sellers: Vec<SellerQuote>,
    pub subtotal_cents: i64, pub discount_cents: i64,
    pub shipping_cents: i64, pub total_cents: i64,
}

pub struct CheckoutQuoteUseCase { db: DbConn }
impl CheckoutQuoteUseCase {
    pub fn new(db: DbConn) -> Self { Self { db } }

    pub async fn create(&self, user_id: i64, request: QuoteRequest) -> Result<QuoteResult, PurchaseError> {
        let customer = PurchaseGateway::customer_for_user(&self.db, user_id).await?;
        let mut quote = Self::calculate(&self.db, customer.id, &request).await?;
        let expiry = Utc::now().naive_utc() + Duration::minutes(15);
        let record = checkout_quote_entity::ActiveModel {
            customer_id: Set(customer.id),
            request: Set(serde_json::to_value(&request).map_err(|_| PurchaseError::Validation("invalid quote"))?),
            result: Set(serde_json::to_value(&quote).map_err(|_| PurchaseError::Validation("invalid quote"))?),
            expires_at: Set(expiry), created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        }.insert(&self.db).await?;
        quote.id = record.id;
        quote.expires_at = expiry;
        Ok(quote)
    }

    pub async fn load<C: ConnectionTrait>(db: &C, customer_id: i64, id: i64) -> Result<(QuoteRequest, QuoteResult), PurchaseError> {
        let (request, result) = Self::load_any(db, customer_id, id).await?;
        if result.expires_at <= Utc::now().naive_utc() { return Err(PurchaseError::Conflict); }
        Ok((request, result))
    }

    pub async fn load_any<C: ConnectionTrait>(db: &C, customer_id: i64, id: i64) -> Result<(QuoteRequest, QuoteResult), PurchaseError> {
        let record = checkout_quote_entity::Entity::find_by_id(id).one(db).await?
            .ok_or(PurchaseError::NotFound)?;
        if record.customer_id != customer_id { return Err(PurchaseError::NotFound); }
        let request = serde_json::from_value(record.request).map_err(|_| PurchaseError::Validation("invalid quote"))?;
        let mut result: QuoteResult = serde_json::from_value(record.result).map_err(|_| PurchaseError::Validation("invalid quote"))?;
        result.id = id;
        result.expires_at = record.expires_at;
        Ok((request, result))
    }

    pub async fn calculate<C: ConnectionTrait>(db: &C, customer_id: i64, request: &QuoteRequest) -> Result<QuoteResult, PurchaseError> {
        if request.address_id.is_some() == request.shipping_address.is_some() {
            return Err(PurchaseError::Validation("choose one delivery address"));
        }
        let address = if let Some(id) = request.address_id {
            let saved = customer_address_entity::Entity::find_by_id(id).one(db).await?
                .filter(|a| a.customer_id == customer_id).ok_or(PurchaseError::NotFound)?;
            AddressInput {
                recipient: saved.recipient,
                address_line1: saved.address_line1.unwrap_or_default(),
                address_line2: saved.address_line2,
                locality: saved.locality.unwrap_or_default(),
                administrative_area: saved.administrative_area.unwrap_or_default(),
                postal_code: saved.postal_code.unwrap_or_default(),
                country_code: saved.country_code.unwrap_or_default(),
            }
        } else { request.shipping_address.clone().expect("checked address") };
        let normalized = CreatePurchaseInput { items: request.items.clone(), shipping_address: address, billing_address: None }.normalize()?;
        let address = normalized.shipping_address;
        if address.country_code != "BR" || address.administrative_area.len() != 2 || !address.administrative_area.bytes().all(|c| c.is_ascii_uppercase()) {
            return Err(PurchaseError::Validation("Brazilian delivery address required"));
        }
        let mut items = Vec::new();
        let mut seller_subtotals = BTreeMap::<i64, i64>::new();
        for line in normalized.items {
            let (sku, product, _) = PurchaseGateway::catalog(db, line.sku_id).await?;
            let tenant_id = sku.tenant_id.ok_or(PurchaseError::Validation("seller required"))?;
            let total_cents = i64::from(sku.price_cents).checked_mul(i64::from(line.quantity)).ok_or(PurchaseError::Validation("amount overflow"))?;
            *seller_subtotals.entry(tenant_id).or_default() = seller_subtotals.get(&tenant_id).copied().unwrap_or(0)
                .checked_add(total_cents).ok_or(PurchaseError::Validation("amount overflow"))?;
            items.push(QuoteItem { sku_id: sku.id, tenant_id, name: product.name, quantity: line.quantity,
                unit_price_cents: i64::from(sku.price_cents), total_cents });
        }
        let mut codes = BTreeMap::new();
        for coupon in &request.coupons {
            if coupon.code.trim().is_empty() || coupon.code.len() > 40 || codes.insert(coupon.tenant_id, coupon.code.trim().to_uppercase()).is_some() {
                return Err(PurchaseError::Validation("invalid seller coupon"));
            }
        }
        if codes.keys().any(|id| !seller_subtotals.contains_key(id)) { return Err(PurchaseError::Validation("coupon seller absent")); }
        let mut sellers = Vec::new();
        let mut totals = (0_i64, 0_i64, 0_i64, 0_i64);
        for (tenant_id, subtotal_cents) in seller_subtotals {
            let shipping = shipping_rate_entity::Entity::find()
                .filter(shipping_rate_entity::Column::TenantId.eq(tenant_id))
                .filter(shipping_rate_entity::Column::Uf.eq(&address.administrative_area))
                .one(db).await?;
            let shipping = match shipping {
                Some(s) => s,
                None => {
                    let tenant = entity::tenant_entity::Entity::find_by_id(tenant_id).one(db).await?;
                    let seller_name = tenant.map(|t| t.business_name).unwrap_or_else(|| format!("Vendedor {}", tenant_id));
                    return Err(PurchaseError::DeliveryRateMissing(format!(
                        "O vendedor {} não entrega para o estado {}.",
                        seller_name, address.administrative_area
                    )));
                }
            };
            if shipping.price_cents < 0 { return Err(PurchaseError::Validation("invalid shipping rate")); }
            let mut discount_cents = 0_i64;
            let mut coupon_id = None;
            let mut coupon_code = None;
            if let Some(code) = codes.get(&tenant_id) {
                let coupon = coupon_entity::Entity::find()
                    .filter(coupon_entity::Column::TenantId.eq(tenant_id))
                    .filter(coupon_entity::Column::Code.eq(code))
                    .filter(coupon_entity::Column::Active.eq(true))
                    .one(db).await?.ok_or(PurchaseError::Validation("coupon unavailable"))?;
                let now = Utc::now().naive_utc();
                if coupon.starts_at.is_some_and(|v| v > now) || coupon.expires_at.is_some_and(|v| v <= now)
                    || coupon.min_order_cents.is_some_and(|v| subtotal_cents < i64::from(v)) {
                    return Err(PurchaseError::Validation("coupon unavailable"));
                }
                let redeemed = coupon_redemption_entity::Entity::find()
                    .filter(coupon_redemption_entity::Column::CouponId.eq(coupon.id));
                let reserved = db.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres,
                    "SELECT COUNT(*) AS total, COUNT(*) FILTER (WHERE customer_id=$2) AS customer FROM checkout_coupon_reservation WHERE coupon_id=$1 AND status='reserved' AND expires_at > CURRENT_TIMESTAMP",
                    [coupon.id.into(), customer_id.into()])).await?.ok_or(PurchaseError::Validation("reservation count failed"))?;
                let total_uses = redeemed.clone().count(db).await? + reserved.try_get::<i64>("", "total")? as u64;
                let customer_uses = redeemed.filter(coupon_redemption_entity::Column::CustomerId.eq(customer_id)).count(db).await?
                    + reserved.try_get::<i64>("", "customer")? as u64;
                if coupon.max_uses.is_some_and(|v| total_uses >= v as u64) || coupon.max_uses_per_customer.is_some_and(|v| customer_uses >= v as u64) {
                    return Err(PurchaseError::Validation("coupon exhausted"));
                }
                discount_cents = match coupon.coupon_type.as_str() {
                    "PERCENTAGE" if (0..=100).contains(&coupon.value) => (subtotal_cents * i64::from(coupon.value) + 50) / 100,
                    "FIXED" if coupon.value >= 0 => i64::from(coupon.value),
                    _ => return Err(PurchaseError::Validation("invalid coupon")),
                }.min(subtotal_cents);
                coupon_id = Some(coupon.id);
                coupon_code = Some(coupon.code);
            }
            let shipping_cents = i64::from(shipping.price_cents);
            let total_cents = subtotal_cents - discount_cents + shipping_cents;
            totals.0 = totals.0.checked_add(subtotal_cents).ok_or(PurchaseError::Validation("amount overflow"))?;
            totals.1 = totals.1.checked_add(discount_cents).ok_or(PurchaseError::Validation("amount overflow"))?;
            totals.2 = totals.2.checked_add(shipping_cents).ok_or(PurchaseError::Validation("amount overflow"))?;
            totals.3 = totals.3.checked_add(total_cents).ok_or(PurchaseError::Validation("amount overflow"))?;
            sellers.push(SellerQuote { tenant_id, subtotal_cents, discount_cents, shipping_cents, total_cents, coupon_id, coupon_code });
        }
        if sellers.is_empty() { return Err(PurchaseError::Validation("empty cart")); }
        Ok(QuoteResult { id: 0, expires_at: Utc::now().naive_utc(), shipping_address: address,
            items, sellers, subtotal_cents: totals.0, discount_cents: totals.1,
            shipping_cents: totals.2, total_cents: totals.3 })
    }
}
