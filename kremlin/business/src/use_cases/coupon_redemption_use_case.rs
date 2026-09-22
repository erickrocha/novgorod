use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::coupon_redemption::{CouponRedemption, CouponRedemptionEntityMapper};
use crate::gateway::coupon_redemption_gateway::CouponRedemptionGateway;

pub struct CouponRedemptionUseCase {
    gateway: CouponRedemptionGateway,
}

impl CouponRedemptionUseCase {
    pub fn new(gateway: CouponRedemptionGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(
        &self,
        redemption: CouponRedemption,
    ) -> Option<CouponRedemption> {
        let entity = self.gateway.persist(redemption).await.map_err(|e| {
            log::error!("Failed to persist coupon redemption: {}", e);
        }).ok()?;
        Some(CouponRedemptionEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<CouponRedemption> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        CouponRedemptionEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<CouponRedemption> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CouponRedemptionEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<CouponRedemption> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(CouponRedemptionEntityMapper::from_model(entity))
    }

    pub async fn find_by_coupon_id(
        &self,
        coupon_id: i64,
    ) -> Vec<CouponRedemption> {
        let entities = self
            .gateway
            .find_by_coupon_id(coupon_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        CouponRedemptionEntityMapper::from_models(entities)
    }

    pub async fn update(
        &self,
        id: i64,
        mut redemption: CouponRedemption,
    ) -> Option<CouponRedemption> {
        redemption.id = Some(id);
        let entity = self.gateway.persist(redemption).await.map_err(|e| {
            log::error!("Failed to update coupon redemption: {}", e);
        }).ok()?;
        Some(CouponRedemptionEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete coupon redemption: {}", e);
        }).ok()?;
        Some(())
    }
}
