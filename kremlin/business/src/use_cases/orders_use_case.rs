use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::orders::{Orders, OrdersEntityMapper};
use crate::gateway::orders_gateway::OrdersGateway;

pub struct OrdersUseCase {
    gateway: OrdersGateway,
}

impl OrdersUseCase {
    pub fn new(gateway: OrdersGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, orders: Orders) -> Option<Orders> {
        let entity = self.gateway.persist(orders).await.map_err(|e| {
            log::error!("Failed to persist order: {}", e);
        }).ok()?;
        Some(OrdersEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<Orders> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        OrdersEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Orders> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(OrdersEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Orders> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(OrdersEntityMapper::from_model(entity))
    }

    pub async fn find_by_number(&self, number: String) -> Option<Orders> {
        let entity = self.gateway.find_by_number(number).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(OrdersEntityMapper::from_model(entity))
    }

    pub async fn find_by_customer_id(&self, customer_id: i64) -> Vec<Orders> {
        let entities = self
            .gateway
            .find_by_customer_id(customer_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        OrdersEntityMapper::from_models(entities)
    }

    pub async fn update(&self, id: i64, mut orders: Orders) -> Option<Orders> {
        orders.id = Some(id);
        let entity = self.gateway.persist(orders).await.map_err(|e| {
            log::error!("Failed to update order: {}", e);
        }).ok()?;
        Some(OrdersEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete order: {}", e);
        }).ok()?;
        Some(())
    }
}
