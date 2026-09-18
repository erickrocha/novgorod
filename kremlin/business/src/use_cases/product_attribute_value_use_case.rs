use crate::domain::business_error::BusinessError;
use crate::domain::catalog_attribute_value::CatalogAttributeValue;
use crate::gateway::product_attribute_value_gateway::ProductAttributeValueGateway;

pub struct ProductAttributeValueUseCase {
    gateway: ProductAttributeValueGateway,
}
impl ProductAttributeValueUseCase {
    pub fn new(gateway: ProductAttributeValueGateway) -> Self {
        Self { gateway }
    }
    pub async fn find_all(&self) -> Result<Vec<CatalogAttributeValue>, BusinessError> {
        self.gateway
            .find_all()
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))
    }
}
