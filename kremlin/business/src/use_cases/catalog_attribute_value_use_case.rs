use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::catalog_attribute_value::{
    CatalogAttributeValue, CatalogAttributeValueEntityMapper,
};
use crate::gateway::catalog_attribute_value_gateway::CatalogAttributeValueGateway;

pub struct CatalogAttributeValueUseCase {
    gateway: CatalogAttributeValueGateway,
}

impl CatalogAttributeValueUseCase {
    pub fn new(gateway: CatalogAttributeValueGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(
        &self,
        val: CatalogAttributeValue,
    ) -> Option<CatalogAttributeValue> {
        let entity = self.gateway.persist(val).await.map_err(|e| {
            log::error!("Failed to persist catalog attribute value: {}", e);
        }).ok()?;
        Some(CatalogAttributeValueEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<CatalogAttributeValue> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        CatalogAttributeValueEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<CatalogAttributeValue> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CatalogAttributeValueEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<CatalogAttributeValue> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CatalogAttributeValueEntityMapper::from_model(entity))
    }

    pub async fn find_by_attribute_id(
        &self,
        attribute_id: i64,
    ) -> Vec<CatalogAttributeValue> {
        let entities = self
            .gateway
            .find_by_attribute_id(attribute_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        CatalogAttributeValueEntityMapper::from_models(entities)
    }

    pub async fn update(
        &self,
        id: i64,
        mut val: CatalogAttributeValue,
    ) -> Option<CatalogAttributeValue> {
        val.id = Some(id);
        let entity = self.gateway.persist(val).await.map_err(|e| {
            log::error!("Failed to update catalog attribute value: {}", e);
        }).ok()?;
        Some(CatalogAttributeValueEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete catalog attribute value: {}", e);
        }).ok()?;
        Some(())
    }
}
