use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::province::{Province, ProvinceEntityMapper};
use crate::gateway::province_gateway::ProvinceGateway;

pub struct ProvinceUseCase {
    gateway: ProvinceGateway,
}

impl ProvinceUseCase {
    pub fn new(gateway: ProvinceGateway) -> Self {
        Self { gateway }
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Province> {
        let model = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("[ProvinceUseCase::find_by_id] Database error: {}", e);
        }).ok()??;

        Some(ProvinceEntityMapper::from_model(model))
    }

    pub async fn save(&self, province: Province) -> Option<Province> {
        let model = self
            .gateway
            .persist(province)
            .await
            .map_err(|e| {
                log::error!("[ProvinceUseCase::save] Database error: {}", e);
            })
            .ok()?;
        Some(ProvinceEntityMapper::from_active_model(model))
    }

    pub async fn find_by_ibge_code(&self, code: &str) -> Option<Province> {
        self.gateway
            .find_by_ibge_code(code)
            .await
            .map_err(|e| {
                log::error!("[ProvinceUseCase::find_by_ibge_code] Database error: {}", e);
            })
            .ok()?
            .map(ProvinceEntityMapper::from_model)
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Province> {
        let model = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("[ProvinceUseCase::find_by_uuid] Database error: {}", e);
        }).ok()??;

        Some(ProvinceEntityMapper::from_model(model))
    }

    pub async fn find_by_country_code(&self, country_code: String) -> Vec<Province> {
        let models = self
            .gateway
            .find_by_country_code(&country_code)
            .await
            .map_err(|e| {
                log::error!("[ProvinceUseCase::find_by_country_code] Database error: {}", e);
            })
            .unwrap_or_default();

        ProvinceEntityMapper::from_models(models)
    }

    pub async fn find_all(&self) -> Vec<Province> {
        let models = self
            .gateway
            .find_all()
            .await
            .map_err(|e| {
                log::error!("[ProvinceUseCase::find_all] Database error: {}", e);
            })
            .unwrap_or_default();
        ProvinceEntityMapper::from_models(models)
    }
}
