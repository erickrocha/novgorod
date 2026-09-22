use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::cart_item::{CartItem, CartItemEntityMapper};
use crate::gateway::cart_item_gateway::CartItemGateway;

pub struct CartItemUseCase {
    gateway: CartItemGateway,
}

impl CartItemUseCase {
    pub fn new(gateway: CartItemGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, item: CartItem) -> Option<CartItem> {
        let entity = self.gateway.persist(item).await.map_err(|e| {
            log::error!("Failed to persist cart item: {}", e);
        }).ok()?;
        Some(CartItemEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<CartItem> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        CartItemEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<CartItem> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CartItemEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<CartItem> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CartItemEntityMapper::from_model(entity))
    }

    pub async fn find_by_cart_id(&self, cart_id: i64) -> Vec<CartItem> {
        let entities = self
            .gateway
            .find_by_cart_id(cart_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        CartItemEntityMapper::from_models(entities)
    }

    pub async fn update(&self, id: i64, mut item: CartItem) -> Option<CartItem> {
        item.id = Some(id);
        let entity = self.gateway.persist(item).await.map_err(|e| {
            log::error!("Failed to update cart item: {}", e);
        }).ok()?;
        Some(CartItemEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete cart item: {}", e);
        }).ok()?;
        Some(())
    }
}
