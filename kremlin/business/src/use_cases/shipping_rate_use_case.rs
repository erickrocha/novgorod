use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::shipping_rate::{ShippingRate, ShippingRateEntityMapper};
use crate::gateway::shipping_rate_gateway::ShippingRateGateway;

pub struct ShippingRateUseCase {
    gateway: ShippingRateGateway,
}

impl ShippingRateUseCase {
    pub fn new(gateway: ShippingRateGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, shipping_rate: ShippingRate) -> Result<ShippingRate, BusinessError> {
        let entity = self.gateway.persist(shipping_rate).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist shipping rate: {}", e))
        })?;
        Ok(ShippingRateEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<ShippingRate>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(ShippingRateEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<ShippingRate, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(ShippingRateEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Shipping rate not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<ShippingRate, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(ShippingRateEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Shipping rate not found".to_string())),
        }
    }

    pub async fn update(
        &self,
        id: i64,
        mut shipping_rate: ShippingRate,
    ) -> Result<ShippingRate, BusinessError> {
        shipping_rate.id = Some(id);
        let entity = self.gateway.persist(shipping_rate).await.map_err(|e| {
            BusinessError::new(format!("Failed to update shipping rate: {}", e))
        })?;
        Ok(ShippingRateEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete shipping rate: {}", e))
        })?;
        Ok(())
    }
}
