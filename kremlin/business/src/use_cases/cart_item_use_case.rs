use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::cart_item::{CartItem, CartItemEntityMapper};
use crate::gateway::cart_item_gateway::CartItemGateway;

pub struct CartItemUseCase {
    gateway: CartItemGateway,
}

impl CartItemUseCase {
    pub fn new(gateway: CartItemGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, item: CartItem) -> Result<CartItem, BusinessError> {
        let entity = self.gateway.persist(item).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist cart item: {}", e))
        })?;
        Ok(CartItemEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<CartItem>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(CartItemEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<CartItem, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CartItemEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Cart item not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<CartItem, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CartItemEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Cart item not found".to_string())),
        }
    }

    pub async fn find_by_cart_id(&self, cart_id: i64) -> Result<Vec<CartItem>, BusinessError> {
        let entities = self
            .gateway
            .find_by_cart_id(cart_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(CartItemEntityMapper::from_models(entities))
    }

    pub async fn update(&self, id: i64, mut item: CartItem) -> Result<CartItem, BusinessError> {
        item.id = Some(id);
        let entity = self.gateway.persist(item).await.map_err(|e| {
            BusinessError::new(format!("Failed to update cart item: {}", e))
        })?;
        Ok(CartItemEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete cart item: {}", e))
        })?;
        Ok(())
    }
}
