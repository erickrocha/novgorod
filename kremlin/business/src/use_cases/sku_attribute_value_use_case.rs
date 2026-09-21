use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
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

    pub async fn create(
        &self,
        sav: SkuAttributeValue,
    ) -> Result<SkuAttributeValue, BusinessError> {
        let entity = self.gateway.persist(sav).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist SKU attribute value: {}", e))
        })?;
        Ok(SkuAttributeValueEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<SkuAttributeValue>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(SkuAttributeValueEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<SkuAttributeValue, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(SkuAttributeValueEntityMapper::from_model(value)),
            None => Err(BusinessError::new("SKU attribute value not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<SkuAttributeValue, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(SkuAttributeValueEntityMapper::from_model(value)),
            None => Err(BusinessError::new("SKU attribute value not found".to_string())),
        }
    }

    pub async fn find_by_sku_id(
        &self,
        sku_id: i64,
    ) -> Result<Vec<SkuAttributeValue>, BusinessError> {
        let entities = self
            .gateway
            .find_by_sku_id(sku_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(SkuAttributeValueEntityMapper::from_models(entities))
    }

    pub async fn find_by_product_id(
        &self,
        product_id: i64,
    ) -> Result<Vec<SkuAttributeValue>, BusinessError> {
        let entities = self
            .gateway
            .find_by_product_id(product_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(SkuAttributeValueEntityMapper::from_models(entities))
    }

    pub async fn update(
        &self,
        id: i64,
        mut sav: SkuAttributeValue,
    ) -> Result<SkuAttributeValue, BusinessError> {
        sav.id = Some(id);
        let entity = self.gateway.persist(sav).await.map_err(|e| {
            BusinessError::new(format!("Failed to update SKU attribute value: {}", e))
        })?;
        Ok(SkuAttributeValueEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete SKU attribute value: {}", e))
        })?;
        Ok(())
    }
}
