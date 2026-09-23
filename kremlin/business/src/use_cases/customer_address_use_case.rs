use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::customer_address::{CustomerAddress, CustomerAddressEntityMapper};
use crate::gateway::customer_address_gateway::CustomerAddressGateway;

pub struct CustomerAddressUseCase {
    gateway: CustomerAddressGateway,
}

impl CustomerAddressUseCase {
    pub fn new(gateway: CustomerAddressGateway) -> Self {
        Self { gateway }
    }

    pub async fn persist(&self, address: CustomerAddress) -> Option<CustomerAddress> {
        let entity = self.gateway.persist(address).await.map_err(|e| {
            log::error!("Failed to persist customer address: {}", e);
        }).ok()?;
        Some(CustomerAddressEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<CustomerAddress> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        CustomerAddressEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<CustomerAddress> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CustomerAddressEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<CustomerAddress> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CustomerAddressEntityMapper::from_model(entity))
    }

    pub async fn find_by_customer_id(&self, customer_id: i64) -> Vec<CustomerAddress> {
        let entities = self
            .gateway
            .find_by_customer_id(customer_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        CustomerAddressEntityMapper::from_models(entities)
    }

    pub async fn update(&self, id: i64, mut address: CustomerAddress) -> Option<CustomerAddress> {
        address.id = Some(id);
        let entity = self.gateway.persist(address).await.map_err(|e| {
            log::error!("Failed to update customer address: {}", e);
        }).ok()?;
        Some(CustomerAddressEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete customer address: {}", e);
        }).ok()?;
        Some(())
    }
}
