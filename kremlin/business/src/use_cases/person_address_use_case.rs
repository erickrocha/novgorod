use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::person_address::{PersonAddress, PersonAddressEntityMapper};
use crate::gateway::person_address_gateway::PersonAddressGateway;

pub struct PersonAddressUseCase {
    gateway: PersonAddressGateway,
}

impl PersonAddressUseCase {
    pub fn new(gateway: PersonAddressGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, address: PersonAddress) -> Option<PersonAddress> {
        let entity = self.gateway.persist(address).await.map_err(|e| {
            log::error!("Failed to persist person address: {}", e);
        }).ok()?;
        Some(PersonAddressEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<PersonAddress> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        PersonAddressEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<PersonAddress> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(PersonAddressEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<PersonAddress> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(PersonAddressEntityMapper::from_model(entity))
    }

    pub async fn find_by_person_id(&self, person_id: i64) -> Vec<PersonAddress> {
        let entities = self
            .gateway
            .find_by_person_id(person_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        PersonAddressEntityMapper::from_models(entities)
    }

    pub async fn update(&self, id: i64, mut address: PersonAddress) -> Option<PersonAddress> {
        address.id = Some(id);
        let entity = self.gateway.persist(address).await.map_err(|e| {
            log::error!("Failed to update person address: {}", e);
        }).ok()?;
        Some(PersonAddressEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete person address: {}", e);
        }).ok()?;
        Some(())
    }
}
