//! Future provider boundary. Purchase persistence never invokes a provider.
use sea_orm::prelude::async_trait::async_trait;

pub struct ChargeRequest {
    pub payment_id: i64,
    pub amount_cents: i64,
    pub currency: String,
    pub idempotency_key: String,
    pub provider_token: String,
    pub external_reference: String,
    pub payer_email: String,
    pub payer_tax_id: String,
    pub payment_method_id: String,
    pub issuer_id: Option<String>,
    pub installments: i32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderStatus {
    Pending,
    Authorized,
    Captured,
    Failed,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCardMetadata {
    pub brand: Option<String>,
    pub last_four_digits: Option<String>,
    pub expiration_month: Option<i32>,
    pub expiration_year: Option<i32>,
    pub cardholder_name: Option<String>,
}

pub struct ProviderResult {
    pub reference: String,
    pub status: ProviderStatus,
    pub amount_cents: i64,
    pub currency: String,
    pub external_reference: String,
    pub collector_id: i64,
    pub payment_type_id: String,
    pub card: Option<ProviderCardMetadata>,
}
#[derive(Debug)]
pub struct ProviderError(pub String);

#[async_trait]
pub trait PaymentProviderGateway: Send + Sync {
    async fn charge(&self, request: ChargeRequest) -> Result<ProviderResult, ProviderError>;
    async fn status(&self, reference: &str) -> Result<ProviderResult, ProviderError>;
    async fn search(&self, _external_reference: &str) -> Result<Option<ProviderResult>, ProviderError> {
        Ok(None)
    }
    /// Voids an unresolved payment; never refunds. The result carries the provider's post-cancel status.
    async fn cancel(&self, _reference: &str) -> Result<ProviderResult, ProviderError> {
        Err(ProviderError("cancel not supported".into()))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/gateway/payment_provider_gateway.rs"]
mod tests;
