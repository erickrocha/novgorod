use crate::domain::business_error::BusinessError;
use crate::domain::product::Product;
use crate::gateway::product_gateway::ProductGateway;

pub struct ProductUseCase { gateway: ProductGateway }
impl ProductUseCase {
    pub fn new(gateway: ProductGateway) -> Self { Self { gateway } }
    pub async fn find_all(&self) -> Result<Vec<Product>, BusinessError> {
        self.gateway.find_all().await.map_err(|e| BusinessError::new(format!("Database error: {}", e)))
    }
}
