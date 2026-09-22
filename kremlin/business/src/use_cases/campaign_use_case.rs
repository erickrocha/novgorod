use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::campaign::{Campaign, CampaignEntityMapper};
use crate::gateway::campaign_gateway::CampaignGateway;

pub struct CampaignUseCase {
    gateway: CampaignGateway,
}

impl CampaignUseCase {
    pub fn new(gateway: CampaignGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, campaign: Campaign) -> Option<Campaign> {
        match self.gateway.persist(campaign).await {
            Ok(entity) => Some(CampaignEntityMapper::from_active_model(entity)),
            Err(e) => {
                log::error!("Failed to persist campaign: {}", e);
                None
            }
        }
    }

    pub async fn find_all(&self) -> Vec<Campaign> {
        match self.gateway.find_all().await {
            Ok(entities) => CampaignEntityMapper::from_models(entities),
            Err(e) => {
                log::error!("Database error: {}", e);
                Vec::new()
            }
        }
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Campaign> {
        match self.gateway.find_by_id(id).await {
            Ok(Some(value)) => Some(CampaignEntityMapper::from_model(value)),
            Ok(None) => None,
            Err(e) => {
                log::error!("Database error: {}", e);
                None
            }
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Campaign> {
        match self.gateway.find_by_uuid(uuid).await {
            Ok(Some(value)) => Some(CampaignEntityMapper::from_model(value)),
            Ok(None) => None,
            Err(e) => {
                log::error!("Database error: {}", e);
                None
            }
        }
    }

    pub async fn update(&self, id: i64, mut campaign: Campaign) -> Option<Campaign> {
        campaign.id = Some(id);
        match self.gateway.persist(campaign).await {
            Ok(entity) => Some(CampaignEntityMapper::from_active_model(entity)),
            Err(e) => {
                log::error!("Failed to update campaign: {}", e);
                None
            }
        }
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        match self.gateway.delete_by_id(id).await {
            Ok(_) => Some(()),
            Err(e) => {
                log::error!("Failed to delete campaign: {}", e);
                None
            }
        }
    }
}
