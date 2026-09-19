use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::tax_rule::{TaxRule, TaxRuleEntityMapper};
use crate::gateway::tax_rule_gateway::TaxRuleGateway;

pub struct TaxRuleUseCase {
    gateway: TaxRuleGateway,
}

impl TaxRuleUseCase {
    pub fn new(gateway: TaxRuleGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, tax_rule: TaxRule) -> Result<TaxRule, BusinessError> {
        let entity = self.gateway.persist(tax_rule).await.map_err(|e| {
            BusinessError::new(format!("Failed to persist tax rule: {}", e))
        })?;
        Ok(TaxRuleEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Result<Vec<TaxRule>, BusinessError> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        Ok(TaxRuleEntityMapper::from_models(entities))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<TaxRule, BusinessError> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(TaxRuleEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Tax rule not found".to_string())),
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<TaxRule, BusinessError> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            BusinessError::new(format!("Database error: {}", e))
        })?;
        match entity {
            Some(value) => Ok(TaxRuleEntityMapper::from_model(value)),
            None => Err(BusinessError::new("Tax rule not found".to_string())),
        }
    }

    pub async fn update(&self, id: i64, mut tax_rule: TaxRule) -> Result<TaxRule, BusinessError> {
        tax_rule.id = Some(id);
        let entity = self.gateway.persist(tax_rule).await.map_err(|e| {
            BusinessError::new(format!("Failed to update tax rule: {}", e))
        })?;
        Ok(TaxRuleEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), BusinessError> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            BusinessError::new(format!("Failed to delete tax rule: {}", e))
        })?;
        Ok(())
    }
}
