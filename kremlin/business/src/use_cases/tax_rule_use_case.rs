use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::tax_rule::{TaxRule, TaxRuleEntityMapper};
use crate::gateway::tax_rule_gateway::TaxRuleGateway;

pub struct TaxRuleUseCase {
    gateway: TaxRuleGateway,
}

impl TaxRuleUseCase {
    pub fn new(gateway: TaxRuleGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, tax_rule: TaxRule) -> Option<TaxRule> {
        let entity = self.gateway.persist(tax_rule).await.map_err(|e| {
            log::error!("Failed to persist tax rule: {}", e);
        }).ok()?;
        Some(TaxRuleEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<TaxRule> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        TaxRuleEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<TaxRule> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(TaxRuleEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<TaxRule> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(TaxRuleEntityMapper::from_model(entity))
    }

    pub async fn update(&self, id: i64, mut tax_rule: TaxRule) -> Option<TaxRule> {
        tax_rule.id = Some(id);
        let entity = self.gateway.persist(tax_rule).await.map_err(|e| {
            log::error!("Failed to update tax rule: {}", e);
        }).ok()?;
        Some(TaxRuleEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete tax rule: {}", e);
        }).ok()?;
        Some(())
    }
}
