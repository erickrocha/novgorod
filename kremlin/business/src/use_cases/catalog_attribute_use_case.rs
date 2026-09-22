use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::catalog_attribute::{CatalogAttribute, CatalogAttributeEntityMapper};
use crate::gateway::catalog_attribute_gateway::CatalogAttributeGateway;

pub struct CatalogAttributeUseCase {
    gateway: CatalogAttributeGateway,
}

impl CatalogAttributeUseCase {
    pub fn new(gateway: CatalogAttributeGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, attr: CatalogAttribute) -> Option<CatalogAttribute> {
        let entity = self.gateway.persist(attr).await.map_err(|e| {
            log::error!("Failed to persist catalog attribute: {}", e);
        }).ok()?;
        Some(CatalogAttributeEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<CatalogAttribute> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        CatalogAttributeEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<CatalogAttribute> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CatalogAttributeEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<CatalogAttribute> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CatalogAttributeEntityMapper::from_model(entity))
    }

    pub async fn update(&self, id: i64, mut attr: CatalogAttribute) -> Option<CatalogAttribute> {
        attr.id = Some(id);
        let entity = self.gateway.persist(attr).await.map_err(|e| {
            log::error!("Failed to update catalog attribute: {}", e);
        }).ok()?;
        Some(CatalogAttributeEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete catalog attribute: {}", e);
        }).ok()?;
        Some(())
    }
}
