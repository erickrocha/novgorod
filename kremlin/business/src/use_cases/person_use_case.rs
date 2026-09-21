use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::person::{Person, PersonEntityMapper};
use crate::gateway::person_gateway::PersonGateway;

pub struct PersonUseCase {
    gateway: PersonGateway,
}

impl PersonUseCase {
    pub fn new(gateway: PersonGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, person: Person) -> Result<Person, BusinessError> {
        let entity = self.gateway.persist(person).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist person: {}", e))
        })?;
        Ok(PersonEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<Person>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(PersonEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Person, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(PersonEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Person not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<Person, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(PersonEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Person not found".to_string())),
        }
    }

    pub async fn find_by_user_id(&self, user_id: i64) -> Result<Option<Person>, BusinessError> {
        let entity = self.gateway.find_by_user_id(user_id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(entity.map(PersonEntityMapper::from_model))
    }

    pub async fn update(&self, id: i64, mut person: Person) -> Result<Person, BusinessError> {
        person.id = Some(id);
        let entity = self.gateway.persist(person).await.map_err(|e| {
            BusinessError::new(format!("Failed to update person: {}", e))
        })?;
        Ok(PersonEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete person: {}", e))
        })?;
        Ok(())
    }
}
