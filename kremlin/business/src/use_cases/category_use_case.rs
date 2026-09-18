use crate::commons::entity_mapper::EntityMapper;
use crate::domain::business_error::BusinessError;
use crate::domain::category::Category;
use crate::domain::category::CategoryEntityMapper;
use crate::gateway::category_gateway::CategoryGateway;

pub struct CategoryUseCase {
    gateway: CategoryGateway,
}
impl CategoryUseCase {
    pub fn new(gateway: CategoryGateway) -> Self {
        Self { gateway }
    }
    pub async fn find_all(&self) -> Result<Vec<Category>, BusinessError> {
        let models = self
            .gateway
            .find_all()
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))?;
        Ok(CategoryEntityMapper::from_models(models))
    }
}
