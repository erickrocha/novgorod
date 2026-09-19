use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::sku::{Sku, SkuEntityMapper};
use crate::gateway::sku_gateway::SkuGateway;

pub struct SkuUseCase {
    gateway: SkuGateway,
}

impl SkuUseCase {
    pub fn new(gateway: SkuGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, sku: Sku) -> Result<Sku, BusinessError> {
        let entity = self.gateway.persist(sku).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist sku: {}", e))
        })?;
        Ok(SkuEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<Sku>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(SkuEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Sku, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(SkuEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Sku not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<Sku, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(SkuEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Sku not found".to_string())),
        }
    }

    pub async fn find_by_code(&self, code: String) -> Result<Sku, BusinessError> {
        let entity = self.gateway.find_by_code(code).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(SkuEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Sku not found".to_string())),
        }
    }

    pub async fn find_by_product_id(&self, product_id: i64) -> Result<Vec<Sku>, BusinessError> {
        let entities = self
            .gateway
            .find_by_product_id(product_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(SkuEntityMapper::from_models(entities))
    }

    pub async fn update(&self, id: i64, mut sku: Sku) -> Result<Sku, BusinessError> {
        sku.id = Some(id);
        let entity = self.gateway.persist(sku).await.map_err(|e| {
            BusinessError::new(format!("Failed to update sku: {}", e))
        })?;
        Ok(SkuEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete sku: {}", e))
        })?;
        Ok(())
    }
}
