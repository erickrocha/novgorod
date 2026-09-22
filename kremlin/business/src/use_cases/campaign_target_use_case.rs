use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::campaign_target::{CampaignTarget, CampaignTargetEntityMapper};
use crate::gateway::campaign_target_gateway::CampaignTargetGateway;

pub struct CampaignTargetUseCase {
    gateway: CampaignTargetGateway,
}

impl CampaignTargetUseCase {
    pub fn new(gateway: CampaignTargetGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, target: CampaignTarget) -> Option<CampaignTarget> {
        match self.gateway.persist(target).await {
            Ok(entity) => Some(CampaignTargetEntityMapper::from_active_model(entity)),
            Err(e) => {
                log::error!("Failed to persist campaign target: {}", e);
                None
            }
        }
    }

    pub async fn find_all(&self) -> Vec<CampaignTarget> {
        match self.gateway.find_all().await {
            Ok(entities) => CampaignTargetEntityMapper::from_models(entities),
            Err(e) => {
                log::error!("Database error: {}", e);
                Vec::new()
            }
        }
    }

    pub async fn find_by_id(&self, id: i64) -> Option<CampaignTarget> {
        match self.gateway.find_by_id(id).await {
            Ok(Some(value)) => Some(CampaignTargetEntityMapper::from_model(value)),
            Ok(None) => None,
            Err(e) => {
                log::error!("Database error: {}", e);
                None
            }
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<CampaignTarget> {
        match self.gateway.find_by_uuid(uuid).await {
            Ok(Some(value)) => Some(CampaignTargetEntityMapper::from_model(value)),
            Ok(None) => None,
            Err(e) => {
                log::error!("Database error: {}", e);
                None
            }
        }
    }

    pub async fn find_by_campaign_id(&self, campaign_id: i64) -> Vec<CampaignTarget> {
        match self.gateway.find_by_campaign_id(campaign_id).await {
            Ok(entities) => CampaignTargetEntityMapper::from_models(entities),
            Err(e) => {
                log::error!("Database error: {}", e);
                Vec::new()
            }
        }
    }

    pub async fn update(&self, id: i64, mut target: CampaignTarget) -> Option<CampaignTarget> {
        target.id = Some(id);
        match self.gateway.persist(target).await {
            Ok(entity) => Some(CampaignTargetEntityMapper::from_active_model(entity)),
            Err(e) => {
                log::error!("Failed to update campaign target: {}", e);
                None
            }
        }
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        match self.gateway.delete_by_id(id).await {
            Ok(_) => Some(()),
            Err(e) => {
                log::error!("Failed to delete campaign target: {}", e);
                None
            }
        }
    }
}
