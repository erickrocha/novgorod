use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::sku_stock::{SkuStock, SkuStockEntityMapper};
use crate::gateway::sku_stock_gateway::SkuStockGateway;

pub struct SkuStockUseCase {
    gateway: SkuStockGateway,
}

impl SkuStockUseCase {
    pub fn new(gateway: SkuStockGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, stock: SkuStock) -> Option<SkuStock> {
        let entity = self.gateway.persist(stock).await.map_err(|e| {
            log::error!("Failed to persist SKU stock: {}", e);
        }).ok()?;
        Some(SkuStockEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<SkuStock> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        SkuStockEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<SkuStock> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(SkuStockEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<SkuStock> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(SkuStockEntityMapper::from_model(entity))
    }

    pub async fn find_by_sku_id(&self, sku_id: i64) -> Option<SkuStock> {
        let entity = self
            .gateway
            .find_by_sku_id(sku_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .ok()??;
        Some(SkuStockEntityMapper::from_model(entity))
    }

    pub async fn update(&self, id: i64, mut stock: SkuStock) -> Option<SkuStock> {
        stock.id = Some(id);
        let entity = self.gateway.persist(stock).await.map_err(|e| {
            log::error!("Failed to update SKU stock: {}", e);
        }).ok()?;
        Some(SkuStockEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete SKU stock: {}", e);
        }).ok()?;
        Some(())
    }
}
