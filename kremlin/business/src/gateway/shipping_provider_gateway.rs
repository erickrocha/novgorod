use crate::domain::shipping::{Parcel, ShippingOption, ShippingService};
use sea_orm::prelude::async_trait::async_trait;
#[derive(Debug, Clone)]
pub struct ShippingRequest {
    pub tenant_id: i64,
    pub configuration_version: i64,
    pub origin_cep: String,
    pub destination_cep: String,
    pub parcel: Parcel,
    pub services: Vec<ShippingService>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShippingProviderError {
    Unavailable,
    InvalidResponse,
    Credentials,
    UnsupportedParcel,
    NoDelivery,
}
#[async_trait]
pub trait ShippingProviderGateway: Send + Sync {
    async fn quote(
        &self,
        request: ShippingRequest,
    ) -> Result<Vec<ShippingOption>, ShippingProviderError>;
}
