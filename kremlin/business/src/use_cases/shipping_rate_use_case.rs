use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::shipping_rate::{ShippingRate, ShippingRateEntityMapper};
use crate::gateway::shipping_rate_gateway::ShippingRateGateway;

pub struct ShippingRateUseCase {
    gateway: ShippingRateGateway,
}

impl ShippingRateUseCase {
    pub fn new(gateway: ShippingRateGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, shipping_rate: ShippingRate) -> Option<ShippingRate> {
        let entity = self.gateway.persist(shipping_rate).await.map_err(|e| {
            log::error!("Failed to persist shipping rate: {}", e);
        }).ok()?;
        Some(ShippingRateEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<ShippingRate> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        ShippingRateEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<ShippingRate> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(ShippingRateEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<ShippingRate> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(ShippingRateEntityMapper::from_model(entity))
    }

    pub async fn update(
        &self,
        id: i64,
        mut shipping_rate: ShippingRate,
    ) -> Option<ShippingRate> {
        shipping_rate.id = Some(id);
        let entity = self.gateway.persist(shipping_rate).await.map_err(|e| {
            log::error!("Failed to update shipping rate: {}", e);
        }).ok()?;
        Some(ShippingRateEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete shipping rate: {}", e);
        }).ok()?;
        Some(())
    }
}
