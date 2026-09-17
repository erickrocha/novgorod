use crate::domain::business_error::BusinessError;
use crate::gateway::catalog_attribute_gateway::CatalogAttributeGateway;
pub struct CatalogAttributeUseCase {
    gateway: CatalogAttributeGateway,
}
impl CatalogAttributeUseCase {
    pub fn new(gateway: CatalogAttributeGateway) -> Self {
        Self { gateway }
    }
    pub async fn find_all(
        &self,
    ) -> Result<Vec<entity::catalog_attribute_entity::Model>, BusinessError> {
        self.gateway
            .find_all()
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))
    }
}
