//! Future provider boundary. Purchase persistence never invokes a provider.
use sea_orm::prelude::async_trait::async_trait;

pub struct ChargeRequest {
    pub payment_id: i64,
    pub amount_cents: i64,
    pub currency: String,
    pub idempotency_key: String,
    pub provider_token: String,
}
pub enum ProviderStatus {
    Pending,
    Authorized,
    Captured,
    Failed,
}
pub struct ProviderResult {
    pub reference: String,
    pub status: ProviderStatus,
}
#[derive(Debug)]
pub struct ProviderError(pub String);

#[async_trait]
pub trait PaymentProviderGateway: Send + Sync {
    async fn charge(&self, request: ChargeRequest) -> Result<ProviderResult, ProviderError>;
    async fn status(&self, reference: &str) -> Result<ProviderResult, ProviderError>;
}

#[cfg(test)]
#[path = "../../tests/unit/gateway/payment_provider_gateway.rs"]
mod tests;
