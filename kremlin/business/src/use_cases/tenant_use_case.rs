use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::tenant::{Tenant, TenantEntityMapper};
use crate::gateway::tenant_gateway::TenantGateway;

fn valid_country_code(value: &Option<String>) -> bool {
    value
        .as_ref()
        .is_none_or(|code| code.len() == 2 && code.chars().all(|c| c.is_ascii_alphabetic()))
}

pub struct TenantUseCase {
    gateway: TenantGateway,
}

impl TenantUseCase {
    pub fn new(gateway: TenantGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, tenant: Tenant) -> Option<Tenant> {
        log::info!(
            "[TenantUseCase::create] Executing create tenant for business name: {:?}",
            tenant.business_name
        );

        if tenant.business_name.is_empty() {
            log::error!("[TenantUseCase::create] Tenant business name is required");
            return None;
        }
        if !valid_country_code(&tenant.country_code) {
            log::error!("[TenantUseCase::create] Country code must contain two letters");
            return None;
        }

        match self.gateway.persist(tenant).await {
            Ok(entity) => Some(TenantEntityMapper::from_active_model(entity)),
            Err(e) => {
                log::error!("[TenantUseCase::create] Failed to persist tenant: {}", e);
                None
            }
        }
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Tenant> {
        log::info!("[TenantUseCase::find_by_id] Executing for id: {}", id);

        match self.gateway.find_by_id(id).await {
            Ok(Some(value)) => Some(TenantEntityMapper::from_model(value)),
            Ok(None) => {
                log::error!("[TenantUseCase::find_by_id] Tenant not found with id: {}", id);
                None
            }
            Err(e) => {
                log::error!("[TenantUseCase::find_by_id] Database error: {}", e);
                None
            }
        }
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Tenant> {
        log::info!("[TenantUseCase::find_by_uuid] Executing for uuid: {}", uuid);

        match self.gateway.find_by_uuid(uuid.clone()).await {
            Ok(Some(value)) => Some(TenantEntityMapper::from_model(value)),
            Ok(None) => {
                log::error!("[TenantUseCase::find_by_uuid] Tenant not found with uuid: {}", uuid);
                None
            }
            Err(e) => {
                log::error!("[TenantUseCase::find_by_uuid] Database error: {}", e);
                None
            }
        }
    }

    pub async fn find_all(&self) -> Vec<Tenant> {
        log::info!("[TenantUseCase::find_all] Executing find_all tenants");

        match self.gateway.find_all().await {
            Ok(entities) => TenantEntityMapper::from_models(entities),
            Err(e) => {
                log::error!("[TenantUseCase::find_all] Database error: {}", e);
                Vec::new()
            }
        }
    }

    pub async fn update(&self, id: i64, tenant: Tenant) -> Option<Tenant> {
        log::info!(
            "[TenantUseCase::update] Executing update for id {}: {:?}",
            id,
            tenant.business_name
        );

        let existing = match self.find_by_id(id).await {
            Some(t) => t,
            None => {
                log::error!(
                    "[TenantUseCase::update] Tenant to update not found with id {}",
                    id
                );
                return None;
            }
        };

        if tenant
            .company_name
            .as_ref()
            .is_none_or(|n| n.trim().is_empty())
        {
            log::error!("[TenantUseCase::update] Tenant name is required");
            return None;
        }
        if !valid_country_code(&tenant.country_code) {
            log::error!("[TenantUseCase::update] Country code must contain two letters");
            return None;
        }

        let updated_tenant = Tenant {
            id: Some(id),
            uuid: existing.uuid,
            company_name: tenant.company_name,
            business_name: tenant.business_name,
            tax_id: tenant.tax_id,
            email: tenant.email,
            phone: tenant.phone,
            web_site: tenant.web_site,
            address_line1: tenant.address_line1,
            address_line2: tenant.address_line2,
            locality: tenant.locality,
            administrative_area: tenant.administrative_area,
            postal_code: tenant.postal_code,
            country_code: tenant.country_code,
            created_at: existing.created_at,
            created_by: existing.created_by,
            updated_at: None,
            updated_by: tenant.updated_by,
        };

        match self.gateway.persist(updated_tenant).await {
            Ok(entity) => Some(TenantEntityMapper::from_active_model(entity)),
            Err(e) => {
                log::error!("[TenantUseCase::update] Failed to update tenant: {}", e);
                None
            }
        }
    }

    pub async fn persist(&self, tenant: Tenant) -> Option<Tenant> {
        log::info!(
            "[TenantUseCase::persist] Executing persist tenant: {:?}",
            tenant.business_name
        );
        self.create(tenant).await
    }
}

#[cfg(test)]
#[path = "../../tests/unit/use_cases/tenant_use_case.rs"]
mod tests;
