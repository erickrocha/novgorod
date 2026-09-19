use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::coupon::{Coupon, CouponEntityMapper};
use crate::gateway::coupon_gateway::CouponGateway;

pub struct CouponUseCase {
    gateway: CouponGateway,
}

impl CouponUseCase {
    pub fn new(gateway: CouponGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, coupon: Coupon) -> Result<Coupon, BusinessError> {
        let entity = self.gateway.persist(coupon).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist coupon: {}", e))
        })?;
        Ok(CouponEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<Coupon>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(CouponEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Coupon, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CouponEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Coupon not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<Coupon, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CouponEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Coupon not found".to_string())),
        }
    }

    pub async fn find_by_code(&self, code: String) -> Result<Coupon, BusinessError> {
        let entity = self.gateway.find_by_code(code).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CouponEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Coupon not found".to_string())),
        }
    }

    pub async fn update(&self, id: i64, mut coupon: Coupon) -> Result<Coupon, BusinessError> {
        coupon.id = Some(id);
        let entity = self.gateway.persist(coupon).await.map_err(|e| {
            BusinessError::new(format!("Failed to update coupon: {}", e))
        })?;
        Ok(CouponEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete coupon: {}", e))
        })?;
        Ok(())
    }
}
