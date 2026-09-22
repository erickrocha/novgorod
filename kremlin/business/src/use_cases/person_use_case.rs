use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::person::{Person, PersonEntityMapper};
use crate::gateway::person_gateway::PersonGateway;

pub struct PersonUseCase {
    gateway: PersonGateway,
}

impl PersonUseCase {
    pub fn new(gateway: PersonGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, person: Person) -> Option<Person> {
        let entity = self.gateway.persist(person).await.map_err(|e| {
            log::error!("Failed to persist person: {}", e);
        }).ok()?;
        Some(PersonEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<Person> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        PersonEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Person> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(PersonEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Person> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(PersonEntityMapper::from_model(entity))
    }

    pub async fn find_by_user_id(&self, user_id: i64) -> Option<Person> {
        let entity = self.gateway.find_by_user_id(user_id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(PersonEntityMapper::from_model(entity))
    }

    pub async fn update(&self, id: i64, mut person: Person) -> Option<Person> {
        person.id = Some(id);
        let entity = self.gateway.persist(person).await.map_err(|e| {
            log::error!("Failed to update person: {}", e);
        }).ok()?;
        Some(PersonEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete person: {}", e);
        }).ok()?;
        Some(())
    }
}
