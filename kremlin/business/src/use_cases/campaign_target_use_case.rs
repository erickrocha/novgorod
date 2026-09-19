use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::campaign_target::{CampaignTarget, CampaignTargetEntityMapper};
use crate::gateway::campaign_target_gateway::CampaignTargetGateway;

pub struct CampaignTargetUseCase {
    gateway: CampaignTargetGateway,
}

impl CampaignTargetUseCase {
    pub fn new(gateway: CampaignTargetGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, target: CampaignTarget) -> Result<CampaignTarget, BusinessError> {
        let entity = self.gateway.persist(target).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist campaign target: {}", e))
        })?;
        Ok(CampaignTargetEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<CampaignTarget>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(CampaignTargetEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<CampaignTarget, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CampaignTargetEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Campaign target not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<CampaignTarget, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CampaignTargetEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Campaign target not found".to_string())),
        }
    }

    pub async fn find_by_campaign_id(
        &self,
        campaign_id: i64,
    ) -> Result<Vec<CampaignTarget>, BusinessError> {
        let entities = self
            .gateway
            .find_by_campaign_id(campaign_id)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(CampaignTargetEntityMapper::from_models(entities))
    }

    pub async fn update(
        &self,
        id: i64,
        mut target: CampaignTarget,
    ) -> Result<CampaignTarget, BusinessError> {
        target.id = Some(id);
        let entity = self.gateway.persist(target).await.map_err(|e| {
            BusinessError::new(format!("Failed to update campaign target: {}", e))
        })?;
        Ok(CampaignTargetEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete campaign target: {}", e))
        })?;
        Ok(())
    }
}
