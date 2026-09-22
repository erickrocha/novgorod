use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::cart::{Cart, CartEntityMapper};
use crate::gateway::cart_gateway::CartGateway;

pub struct CartUseCase {
    gateway: CartGateway,
}

impl CartUseCase {
    pub fn new(gateway: CartGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, cart: Cart) -> Option<Cart> {
        let entity = self.gateway.persist(cart).await.map_err(|e| {
            log::error!("Failed to persist cart: {}", e);
        }).ok()?;
        Some(CartEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<Cart> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        CartEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Cart> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CartEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Cart> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CartEntityMapper::from_model(entity))
    }

    pub async fn update(&self, id: i64, mut cart: Cart) -> Option<Cart> {
        cart.id = Some(id);
        let entity = self.gateway.persist(cart).await.map_err(|e| {
            log::error!("Failed to update cart: {}", e);
        }).ok()?;
        Some(CartEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete cart: {}", e);
        }).ok()?;
        Some(())
    }
}
