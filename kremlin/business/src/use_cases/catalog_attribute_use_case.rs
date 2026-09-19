use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::catalog_attribute::{CatalogAttribute, CatalogAttributeEntityMapper};
use crate::gateway::catalog_attribute_gateway::CatalogAttributeGateway;

pub struct CatalogAttributeUseCase {
    gateway: CatalogAttributeGateway,
}

impl CatalogAttributeUseCase {
    pub fn new(gateway: CatalogAttributeGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, attr: CatalogAttribute) -> Result<CatalogAttribute, BusinessError> {
        let entity = self.gateway.persist(attr).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist catalog attribute: {}", e))
        })?;
        Ok(CatalogAttributeEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<CatalogAttribute>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(CatalogAttributeEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<CatalogAttribute, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CatalogAttributeEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Catalog attribute not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<CatalogAttribute, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CatalogAttributeEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Catalog attribute not found".to_string())),
        }
    }

    pub async fn update(
        &self,
        id: i64,
        mut attr: CatalogAttribute,
    ) -> Result<CatalogAttribute, BusinessError> {
        attr.id = Some(id);
        let entity = self.gateway.persist(attr).await.map_err(|e| {
            BusinessError::new(format!("Failed to update catalog attribute: {}", e))
        })?;
        Ok(CatalogAttributeEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete catalog attribute: {}", e))
        })?;
        Ok(())
    }
}
