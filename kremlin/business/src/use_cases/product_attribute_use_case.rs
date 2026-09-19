use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::product_attribute::{ProductAttribute, ProductAttributeEntityMapper};
use crate::gateway::product_attribute_gateway::ProductAttributeGateway;

pub struct ProductAttributeUseCase {
    gateway: ProductAttributeGateway,
}

impl ProductAttributeUseCase {
    pub fn new(gateway: ProductAttributeGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(
        &self,
        attr: ProductAttribute,
    ) -> Result<ProductAttribute, BusinessError> {
        let entity = self.gateway.persist(attr).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist product attribute: {}", e))
        })?;
        Ok(ProductAttributeEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<ProductAttribute>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(ProductAttributeEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<ProductAttribute, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(ProductAttributeEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Product attribute not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<ProductAttribute, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(ProductAttributeEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Product attribute not found".to_string())),
        }
    }

    pub async fn find_by_product_id(
        &self,
        product_id: i64,
    ) -> Result<Vec<ProductAttribute>, BusinessError> {
        let entities = self
            .gateway
            .find_by_product_id(product_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(ProductAttributeEntityMapper::from_models(entities))
    }

    pub async fn update(
        &self,
        id: i64,
        mut attr: ProductAttribute,
    ) -> Result<ProductAttribute, BusinessError> {
        attr.id = Some(id);
        let entity = self.gateway.persist(attr).await.map_err(|e| {
            BusinessError::new(format!("Failed to update product attribute: {}", e))
        })?;
        Ok(ProductAttributeEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete product attribute: {}", e))
        })?;
        Ok(())
    }
}
