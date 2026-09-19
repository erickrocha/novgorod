use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
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
    ) -> Result<OrderStatusHistory, BusinessError> {
        let entity = self.gateway.persist(history).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist order status history: {}", e))
        })?;
        Ok(OrderStatusHistoryEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<OrderStatusHistory>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(OrderStatusHistoryEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<OrderStatusHistory, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(OrderStatusHistoryEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Order status history not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<OrderStatusHistory, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(OrderStatusHistoryEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Order status history not found".to_string())),
        }
    }

    pub async fn find_by_order_id(
        &self,
        order_id: i64,
    ) -> Result<Vec<OrderStatusHistory>, BusinessError> {
        let entities = self
            .gateway
            .find_by_order_id(order_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(OrderStatusHistoryEntityMapper::from_models(entities))
    }

    pub async fn update(
        &self,
        id: i64,
        mut history: OrderStatusHistory,
    ) -> Result<OrderStatusHistory, BusinessError> {
        history.id = Some(id);
        let entity = self.gateway.persist(history).await.map_err(|e| {
            BusinessError::new(format!("Failed to update order status history: {}", e))
        })?;
        Ok(OrderStatusHistoryEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete order status history: {}", e))
        })?;
        Ok(())
    }
}
