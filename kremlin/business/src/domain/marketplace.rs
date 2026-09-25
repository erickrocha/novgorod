use super::{
    credit_card_details::CreditCardDetails, order_address::OrderAddress, order_item::OrderItem,
    order_status_history::OrderStatusHistory, orders::Orders, payment::Payment,
    payment_allocation::PaymentAllocation, purchase::Purchase,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum PurchaseError {
    Validation(&'static str),
    NotFound,
    Forbidden,
    Conflict,
    Persistence(sea_orm::DbErr),
}
impl From<sea_orm::DbErr> for PurchaseError {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::Persistence(value)
    }
}
impl std::fmt::Display for PurchaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for PurchaseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PurchaseAccess {
    Customer { customer_id: i64, user_id: i64 },
    Seller(i64),
    Admin,
}
impl PurchaseAccess {
    pub fn can_read_order(self, order: &Orders) -> bool {
        match self {
            Self::Customer { customer_id, .. } => order.customer_id == customer_id,
            Self::Seller(tenant_id) => order.tenant_id == tenant_id,
            Self::Admin => true,
        }
    }
    pub fn can_read_purchase(self, purchase: &Purchase) -> bool {
        match self {
            Self::Customer { customer_id, .. } => purchase.customer_id == customer_id,
            Self::Seller(_) => false,
            Self::Admin => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AddressInput {
    pub recipient: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub locality: String,
    pub administrative_area: String,
    pub postal_code: String,
    pub country_code: String,
}
impl AddressInput {
    fn normalize(mut self) -> Result<Self, PurchaseError> {
        for (value, max) in [
            (&mut self.recipient, 150),
            (&mut self.address_line1, 500),
            (&mut self.locality, 500),
            (&mut self.administrative_area, 500),
            (&mut self.postal_code, 20),
            (&mut self.country_code, 2),
        ] {
            *value = value.trim().to_string();
            if value.is_empty() || value.chars().count() > max {
                return Err(PurchaseError::Validation("invalid address"));
            }
        }
        self.country_code.make_ascii_uppercase();
        if self.country_code.len() != 2
            || !self.country_code.bytes().all(|c| c.is_ascii_uppercase())
        {
            return Err(PurchaseError::Validation("invalid country code"));
        }
        self.address_line2 = self
            .address_line2
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        if self
            .address_line2
            .as_ref()
            .is_some_and(|s| s.chars().count() > 500)
        {
            return Err(PurchaseError::Validation("invalid address"));
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseItemInput {
    pub sku_id: i64,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatePurchaseInput {
    pub items: Vec<PurchaseItemInput>,
    pub shipping_address: AddressInput,
    pub billing_address: Option<AddressInput>,
}
impl CreatePurchaseInput {
    pub fn normalize(self) -> Result<Self, PurchaseError> {
        if self.items.is_empty() || self.items.len() > 500 {
            return Err(PurchaseError::Validation("one to 500 items required"));
        }
        let mut quantities = BTreeMap::<i64, i32>::new();
        for item in self.items {
            if item.sku_id <= 0 || item.quantity <= 0 {
                return Err(PurchaseError::Validation("invalid item"));
            }
            let quantity = quantities.entry(item.sku_id).or_default();
            *quantity = quantity
                .checked_add(item.quantity)
                .ok_or(PurchaseError::Validation("quantity overflow"))?;
        }
        let shipping_address = self.shipping_address.normalize()?;
        let billing_address = Some(
            self.billing_address
                .unwrap_or_else(|| shipping_address.clone())
                .normalize()?,
        );
        Ok(Self {
            items: quantities
                .into_iter()
                .map(|(sku_id, quantity)| PurchaseItemInput { sku_id, quantity })
                .collect(),
            shipping_address,
            billing_address,
        })
    }
    pub fn request_hash(&self) -> String {
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(self).expect("serializable purchase input"))
        )
    }
}

#[derive(Debug, Clone)]
pub struct OrderDetail {
    pub order: Orders,
    pub items: Vec<OrderItem>,
    pub addresses: Vec<OrderAddress>,
    pub allocations: Vec<PaymentAllocation>,
}
#[derive(Debug, Clone)]
pub struct PaymentDetail {
    pub payment: Payment,
    pub card: Option<CreditCardDetails>,
}
#[derive(Debug, Clone)]
pub struct PurchaseDetail {
    pub purchase: Purchase,
    pub orders: Vec<OrderDetail>,
    pub payments: Vec<PaymentDetail>,
}
pub struct CreatedPurchase {
    pub detail: PurchaseDetail,
    pub replayed: bool,
}

#[derive(Debug, Clone)]
pub struct OrderFilter {
    pub offset: u64,
    pub limit: u64,
    pub query: Option<String>,
    pub status: Option<String>,
    pub tenant_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub sort_by: String,
    pub descending: bool,
}
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
}
pub type History = Vec<OrderStatusHistory>;

#[cfg(test)]
#[path = "../../tests/unit/domain/marketplace.rs"]
mod tests;
