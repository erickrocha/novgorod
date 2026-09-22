use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
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
    ) -> Option<ProductAttribute> {
        let entity = self.gateway.persist(attr).await.map_err(|e| {
            log::error!("Failed to persist product attribute: {}", e);
        }).ok()?;
        Some(ProductAttributeEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<ProductAttribute> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        ProductAttributeEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<ProductAttribute> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(ProductAttributeEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<ProductAttribute> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(ProductAttributeEntityMapper::from_model(entity))
    }

    pub async fn find_by_product_id(
        &self,
        product_id: i64,
    ) -> Vec<ProductAttribute> {
        let entities = self
            .gateway
            .find_by_product_id(product_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        ProductAttributeEntityMapper::from_models(entities)
    }

    pub async fn update(
        &self,
        id: i64,
        mut attr: ProductAttribute,
    ) -> Option<ProductAttribute> {
        attr.id = Some(id);
        let entity = self.gateway.persist(attr).await.map_err(|e| {
            log::error!("Failed to update product attribute: {}", e);
        }).ok()?;
        Some(ProductAttributeEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete product attribute: {}", e);
        }).ok()?;
        Some(())
    }
}
