use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::category::{Category, CategoryEntityMapper};
use crate::gateway::category_gateway::CategoryGateway;

pub struct CategoryUseCase {
    gateway: CategoryGateway,
}

impl CategoryUseCase {
    pub fn new(gateway: CategoryGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, category: Category) -> Option<Category> {
        let entity = self.gateway.persist(category).await.map_err(|e| {
            log::error!("Failed to persist category: {}", e);
        }).ok()?;
        Some(CategoryEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<Category> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        CategoryEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Category> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CategoryEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Category> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CategoryEntityMapper::from_model(entity))
    }

    pub async fn update(&self, id: i64, mut category: Category) -> Option<Category> {
        category.id = Some(id);
        let entity = self.gateway.persist(category).await.map_err(|e| {
            log::error!("Failed to update category: {}", e);
        }).ok()?;
        Some(CategoryEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete category: {}", e);
        }).ok()?;
        Some(())
    }
}
