use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::cart::{Cart, CartEntityMapper};
use crate::gateway::cart_gateway::CartGateway;

pub struct CartUseCase {
    gateway: CartGateway,
}

impl CartUseCase {
    pub fn new(gateway: CartGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, cart: Cart) -> Result<Cart, BusinessError> {
        let entity = self.gateway.persist(cart).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist cart: {}", e))
        })?;
        Ok(CartEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<Cart>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(CartEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Cart, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CartEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Cart not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<Cart, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CartEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Cart not found".to_string())),
        }
    }

    pub async fn update(&self, id: i64, mut cart: Cart) -> Result<Cart, BusinessError> {
        cart.id = Some(id);
        let entity = self.gateway.persist(cart).await.map_err(|e| {
            BusinessError::new(format!("Failed to update cart: {}", e))
        })?;
        Ok(CartEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete cart: {}", e))
        })?;
        Ok(())
    }
}
