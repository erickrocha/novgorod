use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::city::{City, CityEntityMapper};
use crate::gateway::city_gateway::CityGateway;

pub struct CityUseCase {
    gateway: CityGateway,
}

impl CityUseCase {
    pub fn new(gateway: CityGateway) -> Self {
        Self { gateway }
    }

    pub async fn find_by_id(&self, id: i64) -> Option<City> {
        let model = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("[CityUseCase::find_by_id] Database error: {}", e);
        }).ok()??;

        Some(CityEntityMapper::from_model(model))
    }

    pub async fn save(&self, city: City) -> Option<City> {
        let model = self
            .gateway
            .persist(city)
            .await
            .map_err(|e| {
                log::error!("[CityUseCase::save] Database error: {}", e);
            })
            .ok()?;
        Some(CityEntityMapper::from_active_model(model))
    }

    pub async fn find_by_ibge_code(&self, code: &str) -> Option<City> {
        self.gateway
            .find_by_ibge_code(code)
            .await
            .map_err(|e| {
                log::error!("[CityUseCase::find_by_ibge_code] Database error: {}", e);
            })
            .ok()?
            .map(CityEntityMapper::from_model)
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<City> {
        let model = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("[CityUseCase::find_by_uuid] Database error: {}", e);
        }).ok()??;

        Some(CityEntityMapper::from_model(model))
    }

    pub async fn find_all(&self) -> Vec<City> {
        let models = self.gateway.find_all().await.map_err(|e| {
            log::error!("[CityUseCase::find_all] Database error: {}", e);
        }).unwrap_or_default();

        CityEntityMapper::from_models(models)
    }

    pub async fn find_by_province_id(&self, province_id: i32) -> Vec<City> {
        let models = self
            .gateway
            .find_by_province_id(province_id)
            .await
            .map_err(|e| {
                log::error!("[CityUseCase::find_by_province_id] Database error: {}", e);
            })
            .unwrap_or_default();

        CityEntityMapper::from_models(models)
    }
}
