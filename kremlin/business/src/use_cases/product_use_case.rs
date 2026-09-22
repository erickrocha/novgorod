use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::product::{Product, ProductEntityMapper};
use crate::gateway::product_gateway::ProductGateway;

pub struct ProductUseCase {
    gateway: ProductGateway,
}

impl ProductUseCase {
    pub fn new(gateway: ProductGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, product: Product) -> Option<Product> {
        let entity = self.gateway.persist(product).await.map_err(|e| {
            log::error!("Failed to persist product: {}", e);
        }).ok()?;
        Some(ProductEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<Product> {
        log::info!("[ProductUseCase::find_all] Executing find all products]");
        let entities = self.gateway.find_all().await;
        match entities {
            Ok(entities) => ProductEntityMapper::from_models(entities),
            Err(e) => {
                log::error!("[ProductUseCase::find_all] Failed to fetch products: {:?}", e.to_string());
                Vec::new()
            }
        }
    }

    pub async fn find_paged_by_cursor(&self, query: crate::domain::product::ProductSearchQuery) -> (Vec<Product>, Option<i64>) {
        log::info!("[ProductUseCase::find_paged_by_cursor] Executing find all paged products]");
        let result = self.gateway.find_paged_by_cursor(query).await;

        match result {
            Ok(entities) => {
                let (domains, next_cursor) = entities;
                let products = ProductEntityMapper::from_models(domains);
                (products, next_cursor)
            }
            Err(e) => {
                log::error!("[ProductUseCase::find_all] Failed to fetch products: {:?}", e.to_string());
                (Vec::new(), None)
            }
        }
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Product> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(ProductEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Product> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(ProductEntityMapper::from_model(entity))
    }

    pub async fn update(&self, id: i64, mut product: Product) -> Option<Product> {
        product.id = Some(id);
        let entity = self.gateway.persist(product).await.map_err(|e| {
            log::error!("Failed to update product: {}", e);
        }).ok()?;
        Some(ProductEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete product: {}", e);
        }).ok()?;
        Some(())
    }
}
