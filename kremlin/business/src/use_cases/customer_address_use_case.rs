use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::customer_address::{CustomerAddress, CustomerAddressEntityMapper};
use crate::gateway::customer_address_gateway::CustomerAddressGateway;

pub struct CustomerAddressUseCase {
    gateway: CustomerAddressGateway,
}

impl CustomerAddressUseCase {
    pub fn new(gateway: CustomerAddressGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, address: CustomerAddress) -> Result<CustomerAddress, BusinessError> {
        let entity = self.gateway.persist(address).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist customer address: {}", e))
        })?;
        Ok(CustomerAddressEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<CustomerAddress>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(CustomerAddressEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<CustomerAddress, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CustomerAddressEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Customer address not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<CustomerAddress, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CustomerAddressEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Customer address not found".to_string())),
        }
    }

    pub async fn find_by_customer_id(
        &self,
        customer_id: i64,
    ) -> Result<Vec<CustomerAddress>, BusinessError> {
        let entities = self
            .gateway
            .find_by_customer_id(customer_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(CustomerAddressEntityMapper::from_models(entities))
    }

    pub async fn update(
        &self,
        id: i64,
        mut address: CustomerAddress,
    ) -> Result<CustomerAddress, BusinessError> {
        address.id = Some(id);
        let entity = self.gateway.persist(address).await.map_err(|e| {
            BusinessError::new(format!("Failed to update customer address: {}", e))
        })?;
        Ok(CustomerAddressEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete customer address: {}", e))
        })?;
        Ok(())
    }
}
