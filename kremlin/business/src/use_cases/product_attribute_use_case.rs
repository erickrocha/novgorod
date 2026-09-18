use crate::domain::business_error::BusinessError;
use crate::domain::product_attribute::ProductAttribute;
use crate::gateway::product_attribute_gateway::ProductAttributeGateway;

pub struct ProductAttributeUseCase {
    gateway: ProductAttributeGateway,
}
impl ProductAttributeUseCase {
    pub fn new(gateway: ProductAttributeGateway) -> Self {
        Self { gateway }
    }
    pub async fn find_all(&self) -> Result<Vec<ProductAttribute>, BusinessError> {
        self.gateway
            .find_all()
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))
    }
}
