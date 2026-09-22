use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::order_status_history::{OrderStatusHistory, OrderStatusHistoryEntityMapper};
use crate::gateway::order_status_history_gateway::OrderStatusHistoryGateway;

pub struct OrderStatusHistoryUseCase {
    gateway: OrderStatusHistoryGateway,
}

impl OrderStatusHistoryUseCase {
    pub fn new(gateway: OrderStatusHistoryGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(
        &self,
        history: OrderStatusHistory,
    ) -> Option<OrderStatusHistory> {
        let entity = self.gateway.persist(history).await.map_err(|e| {
            log::error!("Failed to persist order status history: {}", e);
        }).ok()?;
        Some(OrderStatusHistoryEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<OrderStatusHistory> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        OrderStatusHistoryEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<OrderStatusHistory> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(OrderStatusHistoryEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<OrderStatusHistory> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(OrderStatusHistoryEntityMapper::from_model(entity))
    }

    pub async fn find_by_order_id(
        &self,
        order_id: i64,
    ) -> Vec<OrderStatusHistory> {
        let entities = self
            .gateway
            .find_by_order_id(order_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        OrderStatusHistoryEntityMapper::from_models(entities)
    }

    pub async fn update(
        &self,
        id: i64,
        mut history: OrderStatusHistory,
    ) -> Option<OrderStatusHistory> {
        history.id = Some(id);
        let entity = self.gateway.persist(history).await.map_err(|e| {
            log::error!("Failed to update order status history: {}", e);
        }).ok()?;
        Some(OrderStatusHistoryEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete order status history: {}", e);
        }).ok()?;
        Some(())
    }
}
