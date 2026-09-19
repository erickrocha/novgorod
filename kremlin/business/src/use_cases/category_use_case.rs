use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::category::{Category, CategoryEntityMapper};
use crate::gateway::category_gateway::CategoryGateway;

pub struct CategoryUseCase {
    gateway: CategoryGateway,
}

impl CategoryUseCase {
    pub fn new(gateway: CategoryGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, category: Category) -> Result<Category, BusinessError> {
        let entity = self.gateway.persist(category).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist category: {}", e))
        })?;
        Ok(CategoryEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<Category>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(CategoryEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Category, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CategoryEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Category not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<Category, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CategoryEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Category not found".to_string())),
        }
    }

    pub async fn update(&self, id: i64, mut category: Category) -> Result<Category, BusinessError> {
        category.id = Some(id);
        let entity = self.gateway.persist(category).await.map_err(|e| {
            BusinessError::new(format!("Failed to update category: {}", e))
        })?;
        Ok(CategoryEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete category: {}", e))
        })?;
        Ok(())
    }
}
