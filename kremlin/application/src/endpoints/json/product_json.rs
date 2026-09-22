use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub active: bool,
    pub ncm: String,
    pub cest: Option<String>,
    pub origem_mercadoria: i16,
}