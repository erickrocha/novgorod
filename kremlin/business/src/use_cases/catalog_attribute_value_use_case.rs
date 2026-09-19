use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
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
    ) -> Result<CatalogAttributeValue, BusinessError> {
        let entity = self.gateway.persist(val).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist catalog attribute value: {}", e))
        })?;
        Ok(CatalogAttributeValueEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<CatalogAttributeValue>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(CatalogAttributeValueEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<CatalogAttributeValue, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CatalogAttributeValueEntityMapper::from_model(value)),
            None => Err(BusinessError::new(
                "Catalog attribute value not found".to_string(),
            )),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<CatalogAttributeValue, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CatalogAttributeValueEntityMapper::from_model(value)),
            None => Err(BusinessError::new(
                "Catalog attribute value not found".to_string(),
            )),
        }
    }

    pub async fn find_by_attribute_id(
        &self,
        attribute_id: i64,
    ) -> Result<Vec<CatalogAttributeValue>, BusinessError> {
        let entities = self
            .gateway
            .find_by_attribute_id(attribute_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(CatalogAttributeValueEntityMapper::from_models(entities))
    }

    pub async fn update(
        &self,
        id: i64,
        mut val: CatalogAttributeValue,
    ) -> Result<CatalogAttributeValue, BusinessError> {
        val.id = Some(id);
        let entity = self.gateway.persist(val).await.map_err(|e| {
            BusinessError::new(format!("Failed to update catalog attribute value: {}", e))
        })?;
        Ok(CatalogAttributeValueEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete catalog attribute value: {}", e))
        })?;
        Ok(())
    }
}
