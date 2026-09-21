use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::sku_stock::{SkuStock, SkuStockEntityMapper};
use crate::gateway::sku_stock_gateway::SkuStockGateway;

pub struct SkuStockUseCase {
    gateway: SkuStockGateway,
}

impl SkuStockUseCase {
    pub fn new(gateway: SkuStockGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(
        &self,
        stock: SkuStock,
    ) -> Result<SkuStock, BusinessError> {
        let entity = self.gateway.persist(stock).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist SKU stock: {}", e))
        })?;
        Ok(SkuStockEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<SkuStock>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(SkuStockEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<SkuStock, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(SkuStockEntityMapper::from_model(value)),
            None => Err(BusinessError::new("SKU stock not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<SkuStock, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(SkuStockEntityMapper::from_model(value)),
            None => Err(BusinessError::new("SKU stock not found".to_string())),
        }
    }

    pub async fn find_by_sku_id(
        &self,
        sku_id: i64,
    ) -> Result<Option<SkuStock>, BusinessError> {
        let entity = self
            .gateway
            .find_by_sku_id(sku_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(entity.map(SkuStockEntityMapper::from_model))
    }

    pub async fn update(
        &self,
        id: i64,
        mut stock: SkuStock,
    ) -> Result<SkuStock, BusinessError> {
        stock.id = Some(id);
        let entity = self.gateway.persist(stock).await.map_err(|e| {
            BusinessError::new(format!("Failed to update SKU stock: {}", e))
        })?;
        Ok(SkuStockEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete SKU stock: {}", e))
        })?;
        Ok(())
    }
}
