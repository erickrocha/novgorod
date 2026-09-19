use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::campaign::{Campaign, CampaignEntityMapper};
use crate::gateway::campaign_gateway::CampaignGateway;

pub struct CampaignUseCase {
    gateway: CampaignGateway,
}

impl CampaignUseCase {
    pub fn new(gateway: CampaignGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, campaign: Campaign) -> Result<Campaign, BusinessError> {
        let entity = self.gateway.persist(campaign).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist campaign: {}", e))
        })?;
        Ok(CampaignEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<Campaign>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(CampaignEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Campaign, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CampaignEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Campaign not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<Campaign, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(CampaignEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Campaign not found".to_string())),
        }
    }

    pub async fn update(&self, id: i64, mut campaign: Campaign) -> Result<Campaign, BusinessError> {
        campaign.id = Some(id);
        let entity = self.gateway.persist(campaign).await.map_err(|e| {
            BusinessError::new(format!("Failed to update campaign: {}", e))
        })?;
        Ok(CampaignEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete campaign: {}", e))
        })?;
        Ok(())
    }
}
