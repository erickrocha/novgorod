use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::order_item::{OrderItem, OrderItemEntityMapper};
use crate::gateway::order_item_gateway::OrderItemGateway;

pub struct OrderItemUseCase {
    gateway: OrderItemGateway,
}

impl OrderItemUseCase {
    pub fn new(gateway: OrderItemGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, item: OrderItem) -> Result<OrderItem, BusinessError> {
        let entity = self.gateway.persist(item).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist order item: {}", e))
        })?;
        Ok(OrderItemEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<OrderItem>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(OrderItemEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<OrderItem, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(OrderItemEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Order item not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<OrderItem, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(OrderItemEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Order item not found".to_string())),
        }
    }

    pub async fn find_by_order_id(&self, order_id: i64) -> Result<Vec<OrderItem>, BusinessError> {
        let entities = self
            .gateway
            .find_by_order_id(order_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(OrderItemEntityMapper::from_models(entities))
    }

    pub async fn update(&self, id: i64, mut item: OrderItem) -> Result<OrderItem, BusinessError> {
        item.id = Some(id);
        let entity = self.gateway.persist(item).await.map_err(|e| {
            BusinessError::new(format!("Failed to update order item: {}", e))
        })?;
        Ok(OrderItemEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete order item: {}", e))
        })?;
        Ok(())
    }
}
