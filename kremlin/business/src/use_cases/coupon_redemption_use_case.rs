use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
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
    ) -> Result<CouponRedemption, BusinessError> {
        let entity = self.gateway.persist(redemption).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist coupon redemption: {}", e))
        })?;
        Ok(CouponRedemptionEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<CouponRedemption>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(CouponRedemptionEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<CouponRedemption, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CouponRedemptionEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Coupon redemption not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<CouponRedemption, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CouponRedemptionEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Coupon redemption not found".to_string())),
        }
    }

    pub async fn find_by_coupon_id(
        &self,
        coupon_id: i64,
    ) -> Result<Vec<CouponRedemption>, BusinessError> {
        let entities = self
            .gateway
            .find_by_coupon_id(coupon_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(CouponRedemptionEntityMapper::from_models(entities))
    }

    pub async fn update(
        &self,
        id: i64,
        mut redemption: CouponRedemption,
    ) -> Result<CouponRedemption, BusinessError> {
        redemption.id = Some(id);
        let entity = self.gateway.persist(redemption).await.map_err(|e| {
            BusinessError::new(format!("Failed to update coupon redemption: {}", e))
        })?;
        Ok(CouponRedemptionEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete coupon redemption: {}", e))
        })?;
        Ok(())
    }
}
