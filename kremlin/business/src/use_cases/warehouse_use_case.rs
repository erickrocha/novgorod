use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::warehouse::{Warehouse, WarehouseEntityMapper};
use crate::gateway::warehouse_gateway::WarehouseGateway;

pub struct WarehouseUseCase {
    gateway: WarehouseGateway,
}

impl WarehouseUseCase {
    pub fn new(gateway: WarehouseGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, warehouse: Warehouse) -> Option<Warehouse> {
        let entity = self.gateway.persist(warehouse).await.map_err(|e| {
            log::error!("Failed to persist warehouse: {}", e);
        }).ok()?;
        Some(WarehouseEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<Warehouse> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        WarehouseEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Warehouse> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(WarehouseEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Warehouse> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(WarehouseEntityMapper::from_model(entity))
    }

    pub async fn update(&self, id: i64, mut warehouse: Warehouse) -> Option<Warehouse> {
        warehouse.id = Some(id);
        let entity = self.gateway.persist(warehouse).await.map_err(|e| {
            log::error!("Failed to update warehouse: {}", e);
        }).ok()?;
        Some(WarehouseEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete warehouse: {}", e);
        }).ok()?;
        Some(())
    }
}
