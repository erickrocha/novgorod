use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::business_error::BusinessError;
use crate::domain::province::{Province, ProvinceEntityMapper};
use crate::gateway::province_gateway::ProvinceGateway;

pub struct ProvinceUseCase {
    gateway: ProvinceGateway,
}

impl ProvinceUseCase {
    pub fn new(gateway: ProvinceGateway) -> Self {
        Self { gateway }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Province, BusinessError> {
        log::info!("[ProvinceUseCase::find_by_id] Executing for id: {}", id);
        let model = self.gateway.find_by_id(id).await.map_err(|e| {
            let msg = format!("Database error: {}", e);
            log::error!("[ProvinceUseCase::find_by_id] {}", msg);
            BusinessError::new(msg)
        })?;

        match model {
            Some(val) => Ok(ProvinceEntityMapper::from_model(val)),
            None => {
                let msg = format!("Province not found with id: {}", id);
                log::error!("[ProvinceUseCase::find_by_id] {}", msg);
                Err(BusinessError::new("Province not found".to_string()))
            }
        }
    }

    pub async fn save(&self, province: Province) -> Result<Province, BusinessError> {
        let model = self
            .gateway
            .persist(province)
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {e}")))?;
        Ok(ProvinceEntityMapper::from_active_model(model))
    }

    pub async fn find_by_ibge_code(&self, code: &str) -> Result<Option<Province>, BusinessError> {
        self.gateway
            .find_by_ibge_code(code)
            .await
            .map(|v| v.map(ProvinceEntityMapper::from_model))
            .map_err(|e| BusinessError::new(e.to_string()))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Result<Province, BusinessError> {
        log::info!(
            "[ProvinceUseCase::find_by_uuid] Executing for uuid: {}",
            uuid
        );
        let model = self.gateway.find_by_uuid(uuid.clone()).await.map_err(|e| {
            let msg = format!("Database error: {}", e);
            log::error!("[ProvinceUseCase::find_by_uuid] {}", msg);
            BusinessError::new(msg)
        })?;

        match model {
            Some(val) => Ok(ProvinceEntityMapper::from_model(val)),
            None => {
                let msg = format!("Province not found with uuid: {}", uuid);
                log::error!("[ProvinceUseCase::find_by_uuid] {}", msg);
                Err(BusinessError::new("Province not found".to_string()))
            }
        }
    }

    pub async fn find_by_country_code(
        &self,
        country_code: String,
    ) -> Result<Vec<Province>, BusinessError> {
        log::info!(
            "[ProvinceUseCase::find_by_country_code] Executing for country_code: {}",
            country_code
        );
        let models = self
            .gateway
            .find_by_country_code(&country_code)
            .await
            .map_err(|e| {
                let msg = format!("Database error: {}", e);
                log::error!("[ProvinceUseCase::find_by_country_code] {}", msg);
                BusinessError::new(msg)
            })?;

        Ok(ProvinceEntityMapper::from_models(models))
    }

    pub async fn find_all(&self) -> Result<Vec<Province>, BusinessError> {
        let models = self
            .gateway
            .find_all()
            .await
            .map_err(|e| BusinessError::new(e.to_string()))?;
        Ok(ProvinceEntityMapper::from_models(models))
    }
}
