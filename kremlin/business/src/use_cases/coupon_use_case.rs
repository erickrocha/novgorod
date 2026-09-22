use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::coupon::{Coupon, CouponEntityMapper};
use crate::gateway::coupon_gateway::CouponGateway;

pub struct CouponUseCase {
    gateway: CouponGateway,
}

impl CouponUseCase {
    pub fn new(gateway: CouponGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, coupon: Coupon) -> Option<Coupon> {
        let entity = self.gateway.persist(coupon).await.map_err(|e| {
            log::error!("Failed to persist coupon: {}", e);
        }).ok()?;
        Some(CouponEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<Coupon> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        CouponEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Coupon> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CouponEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Coupon> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CouponEntityMapper::from_model(entity))
    }

    pub async fn find_by_code(&self, code: String) -> Option<Coupon> {
        let entity = self.gateway.find_by_code(code).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CouponEntityMapper::from_model(entity))
    }

    pub async fn update(&self, id: i64, mut coupon: Coupon) -> Option<Coupon> {
        coupon.id = Some(id);
        let entity = self.gateway.persist(coupon).await.map_err(|e| {
            log::error!("Failed to update coupon: {}", e);
        }).ok()?;
        Some(CouponEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete coupon: {}", e);
        }).ok()?;
        Some(())
    }
}
