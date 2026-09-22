use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::product_category::{ProductCategory, ProductCategoryEntityMapper};
use crate::gateway::product_category_gateway::ProductCategoryGateway;

pub struct ProductCategoryUseCase {
    gateway: ProductCategoryGateway,
}

impl ProductCategoryUseCase {
    pub fn new(gateway: ProductCategoryGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, pc: ProductCategory) -> Option<ProductCategory> {
        let entity = self.gateway.persist(pc).await.map_err(|e| {
            log::error!("Failed to persist product category: {}", e);
        }).ok()?;
        Some(ProductCategoryEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<ProductCategory> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        ProductCategoryEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<ProductCategory> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(ProductCategoryEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<ProductCategory> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(ProductCategoryEntityMapper::from_model(entity))
    }

    pub async fn find_by_product_id(&self, product_id: i64) -> Vec<ProductCategory> {
        let entities = self
            .gateway
            .find_by_product_id(product_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        ProductCategoryEntityMapper::from_models(entities)
    }

    pub async fn find_by_category_id(&self, category_id: i64) -> Vec<ProductCategory> {
        let entities = self
            .gateway
            .find_by_category_id(category_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        ProductCategoryEntityMapper::from_models(entities)
    }

    pub async fn find_primary_for_product(&self, product_id: i64) -> Option<ProductCategory> {
        let entity = self
            .gateway
            .find_primary_for_product(product_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .ok()??;
        Some(ProductCategoryEntityMapper::from_model(entity))
    }

    pub async fn update(&self, id: i64, mut pc: ProductCategory) -> Option<ProductCategory> {
        pc.id = Some(id);
        let entity = self.gateway.persist(pc).await.map_err(|e| {
            log::error!("Failed to update product category: {}", e);
        }).ok()?;
        Some(ProductCategoryEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete product category: {}", e);
        }).ok()?;
        Some(())
    }
}
