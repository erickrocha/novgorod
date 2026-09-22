use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::customer::{Customer, CustomerEntityMapper};
use crate::gateway::customer_gateway::CustomerGateway;

pub struct CustomerUseCase {
    gateway: CustomerGateway,
}

impl CustomerUseCase {
    pub fn new(gateway: CustomerGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, customer: Customer) -> Option<Customer> {
        let entity = self.gateway.persist(customer).await.map_err(|e| {
            log::error!("Failed to persist customer: {}", e);
        }).ok()?;
        Some(CustomerEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<Customer> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        CustomerEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Customer> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CustomerEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Customer> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CustomerEntityMapper::from_model(entity))
    }

    pub async fn update(&self, id: i64, mut customer: Customer) -> Option<Customer> {
        customer.id = Some(id);
        let entity = self.gateway.persist(customer).await.map_err(|e| {
            log::error!("Failed to update customer: {}", e);
        }).ok()?;
        Some(CustomerEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete customer: {}", e);
        }).ok()?;
        Some(())
    }
}
