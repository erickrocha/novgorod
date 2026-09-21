use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::person_address::{PersonAddress, PersonAddressEntityMapper};
use crate::gateway::person_address_gateway::PersonAddressGateway;

pub struct PersonAddressUseCase {
    gateway: PersonAddressGateway,
}

impl PersonAddressUseCase {
    pub fn new(gateway: PersonAddressGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, address: PersonAddress) -> Result<PersonAddress, BusinessError> {
        let entity = self.gateway.persist(address).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist person address: {}", e))
        })?;
        Ok(PersonAddressEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<PersonAddress>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(PersonAddressEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<PersonAddress, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(PersonAddressEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Person address not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<PersonAddress, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(PersonAddressEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Person address not found".to_string())),
        }
    }

    pub async fn find_by_person_id(
        &self,
        person_id: i64,
    ) -> Result<Vec<PersonAddress>, BusinessError> {
        let entities = self
            .gateway
            .find_by_person_id(person_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(PersonAddressEntityMapper::from_models(entities))
    }

    pub async fn update(
        &self,
        id: i64,
        mut address: PersonAddress,
    ) -> Result<PersonAddress, BusinessError> {
        address.id = Some(id);
        let entity = self.gateway.persist(address).await.map_err(|e| {
            BusinessError::new(format!("Failed to update person address: {}", e))
        })?;
        Ok(PersonAddressEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete person address: {}", e))
        })?;
        Ok(())
    }
}
