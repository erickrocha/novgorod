use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::sku::{Sku, SkuEntityMapper};
use crate::gateway::sku_gateway::SkuGateway;

pub struct SkuUseCase {
    gateway: SkuGateway,
}

impl SkuUseCase {
    pub fn new(gateway: SkuGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, sku: Sku) -> Option<Sku> {
        let entity = self.gateway.persist(sku).await.map_err(|e| {
            log::error!("Failed to persist sku: {}", e);
        }).ok()?;
        Some(SkuEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<Sku> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        SkuEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Sku> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(SkuEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Sku> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(SkuEntityMapper::from_model(entity))
    }

    pub async fn find_by_code(&self, code: String) -> Option<Sku> {
        let entity = self.gateway.find_by_code(code).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(SkuEntityMapper::from_model(entity))
    }

    pub async fn find_by_product_id(&self, product_id: i64) -> Vec<Sku> {
        let entities = self
            .gateway
            .find_by_product_id(product_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        SkuEntityMapper::from_models(entities)
    }

    pub async fn update(&self, id: i64, mut sku: Sku) -> Option<Sku> {
        sku.id = Some(id);
        let entity = self.gateway.persist(sku).await.map_err(|e| {
            log::error!("Failed to update sku: {}", e);
        }).ok()?;
        Some(SkuEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete sku: {}", e);
        }).ok()?;
        Some(())
    }
}
