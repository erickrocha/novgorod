use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use super::customer_json::{CustomerAddressJson};

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignupRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub cpf: Option<String>,
    pub phone: Option<String>,
    pub address: Option<CustomerAddressJson>,
}
