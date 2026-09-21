use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::product_category::{ProductCategory, ProductCategoryEntityMapper};
use crate::gateway::product_category_gateway::ProductCategoryGateway;

pub struct ProductCategoryUseCase {
    gateway: ProductCategoryGateway,
}

impl ProductCategoryUseCase {
    pub fn new(gateway: ProductCategoryGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(
        &self,
        pc: ProductCategory,
    ) -> Result<ProductCategory, BusinessError> {
        let entity = self.gateway.persist(pc).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist product category: {}", e))
        })?;
        Ok(ProductCategoryEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<ProductCategory>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(ProductCategoryEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<ProductCategory, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(ProductCategoryEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Product category not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<ProductCategory, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(ProductCategoryEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Product category not found".to_string())),
        }
    }

    pub async fn find_by_product_id(
        &self,
        product_id: i64,
    ) -> Result<Vec<ProductCategory>, BusinessError> {
        let entities = self
            .gateway
            .find_by_product_id(product_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(ProductCategoryEntityMapper::from_models(entities))
    }

    pub async fn find_by_category_id(
        &self,
        category_id: i64,
    ) -> Result<Vec<ProductCategory>, BusinessError> {
        let entities = self
            .gateway
            .find_by_category_id(category_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(ProductCategoryEntityMapper::from_models(entities))
    }

    pub async fn find_primary_for_product(
        &self,
        product_id: i64,
    ) -> Result<Option<ProductCategory>, BusinessError> {
        let entity = self
            .gateway
            .find_primary_for_product(product_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(entity.map(ProductCategoryEntityMapper::from_model))
    }

    pub async fn update(
        &self,
        id: i64,
        mut pc: ProductCategory,
    ) -> Result<ProductCategory, BusinessError> {
        pc.id = Some(id);
        let entity = self.gateway.persist(pc).await.map_err(|e| {
            BusinessError::new(format!("Failed to update product category: {}", e))
        })?;
        Ok(ProductCategoryEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete product category: {}", e))
        })?;
        Ok(())
    }
}
