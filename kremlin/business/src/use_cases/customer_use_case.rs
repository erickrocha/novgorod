use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::customer::{Customer, CustomerEntityMapper};
use crate::gateway::customer_gateway::CustomerGateway;

pub struct CustomerUseCase {
    gateway: CustomerGateway,
}

impl CustomerUseCase {
    pub fn new(gateway: CustomerGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, customer: Customer) -> Result<Customer, BusinessError> {
        let entity = self.gateway.persist(customer).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist customer: {}", e))
        })?;
        Ok(CustomerEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<Customer>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(CustomerEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Customer, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CustomerEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Customer not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<Customer, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CustomerEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Customer not found".to_string())),
        }
    }

    pub async fn update(&self, id: i64, mut customer: Customer) -> Result<Customer, BusinessError> {
        customer.id = Some(id);
        let entity = self.gateway.persist(customer).await.map_err(|e| {
            BusinessError::new(format!("Failed to update customer: {}", e))
        })?;
        Ok(CustomerEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete customer: {}", e))
        })?;
        Ok(())
    }
}
