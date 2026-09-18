use crate::domain::business_error::BusinessError;
use crate::domain::sku::Sku;
use crate::gateway::sku_gateway::SkuGateway;

pub struct SkuUseCase {
    gateway: SkuGateway,
}
impl SkuUseCase {
    pub fn new(gateway: SkuGateway) -> Self {
        Self { gateway }
    }
    pub async fn find_all(&self) -> Result<Vec<Sku>, BusinessError> {
        self.gateway
            .find_all()
            .await
            .map_err(|e| BusinessError::new(format!("Database error: {}", e)))
    }
}
