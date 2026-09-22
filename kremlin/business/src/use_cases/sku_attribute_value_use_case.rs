use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::sku_attribute_value::{SkuAttributeValue, SkuAttributeValueEntityMapper};
use crate::gateway::sku_attribute_value_gateway::SkuAttributeValueGateway;

pub struct SkuAttributeValueUseCase {
    gateway: SkuAttributeValueGateway,
}

pub type SkuAttributeUseCase = SkuAttributeValueUseCase;

impl SkuAttributeValueUseCase {
    pub fn new(gateway: SkuAttributeValueGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, sav: SkuAttributeValue) -> Option<SkuAttributeValue> {
        let entity = self.gateway.persist(sav).await.map_err(|e| {
            log::error!("Failed to persist SKU attribute value: {}", e);
        }).ok()?;
        Some(SkuAttributeValueEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<SkuAttributeValue> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        SkuAttributeValueEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<SkuAttributeValue> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(SkuAttributeValueEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<SkuAttributeValue> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(SkuAttributeValueEntityMapper::from_model(entity))
    }

    pub async fn find_by_sku_id(&self, sku_id: i64) -> Vec<SkuAttributeValue> {
        let entities = self
            .gateway
            .find_by_sku_id(sku_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        SkuAttributeValueEntityMapper::from_models(entities)
    }

    pub async fn find_by_product_id(&self, product_id: i64) -> Vec<SkuAttributeValue> {
        let entities = self
            .gateway
            .find_by_product_id(product_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        SkuAttributeValueEntityMapper::from_models(entities)
    }

    pub async fn update(&self, id: i64, mut sav: SkuAttributeValue) -> Option<SkuAttributeValue> {
        sav.id = Some(id);
        let entity = self.gateway.persist(sav).await.map_err(|e| {
            log::error!("Failed to update SKU attribute value: {}", e);
        }).ok()?;
        Some(SkuAttributeValueEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete SKU attribute value: {}", e);
        }).ok()?;
        Some(())
    }
}
