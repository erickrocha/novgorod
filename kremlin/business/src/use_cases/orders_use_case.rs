use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::orders::{Orders, OrdersEntityMapper};
use crate::gateway::orders_gateway::OrdersGateway;

pub struct OrdersUseCase {
    gateway: OrdersGateway,
}

impl OrdersUseCase {
    pub fn new(gateway: OrdersGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, orders: Orders) -> Result<Orders, BusinessError> {
        let entity = self.gateway.persist(orders).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist order: {}", e))
        })?;
        Ok(OrdersEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<Orders>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(OrdersEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Orders, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(OrdersEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Order not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<Orders, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(OrdersEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Order not found".to_string())),
        }
    }

    pub async fn find_by_number(&self, number: String) -> Result<Orders, BusinessError> {
        let entity = self.gateway.find_by_number(number).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(OrdersEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Order not found".to_string())),
        }
    }

    pub async fn find_by_customer_id(&self, customer_id: i64) -> Result<Vec<Orders>, BusinessError> {
        let entities = self
            .gateway
            .find_by_customer_id(customer_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(OrdersEntityMapper::from_models(entities))
    }

    pub async fn update(&self, id: i64, mut orders: Orders) -> Result<Orders, BusinessError> {
        orders.id = Some(id);
        let entity = self.gateway.persist(orders).await.map_err(|e| {
            BusinessError::new(format!("Failed to update order: {}", e))
        })?;
        Ok(OrdersEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete order: {}", e))
        })?;
        Ok(())
    }
}
