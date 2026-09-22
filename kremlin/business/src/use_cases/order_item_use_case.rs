use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::order_item::{OrderItem, OrderItemEntityMapper};
use crate::gateway::order_item_gateway::OrderItemGateway;

pub struct OrderItemUseCase {
    gateway: OrderItemGateway,
}

impl OrderItemUseCase {
    pub fn new(gateway: OrderItemGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, item: OrderItem) -> Option<OrderItem> {
        let entity = self.gateway.persist(item).await.map_err(|e| {
            log::error!("Failed to persist order item: {}", e);
        }).ok()?;
        Some(OrderItemEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<OrderItem> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        OrderItemEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<OrderItem> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(OrderItemEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<OrderItem> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(OrderItemEntityMapper::from_model(entity))
    }

    pub async fn find_by_order_id(&self, order_id: i64) -> Vec<OrderItem> {
        let entities = self
            .gateway
            .find_by_order_id(order_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        OrderItemEntityMapper::from_models(entities)
    }

    pub async fn update(&self, id: i64, mut item: OrderItem) -> Option<OrderItem> {
        item.id = Some(id);
        let entity = self.gateway.persist(item).await.map_err(|e| {
            log::error!("Failed to update order item: {}", e);
        }).ok()?;
        Some(OrderItemEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete order item: {}", e);
        }).ok()?;
        Some(())
    }
}
