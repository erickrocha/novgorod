use crate::commons::pagination::PageQuery;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomerJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub name: String,
    pub email: String,
    pub cpf: Option<String>,
    pub phone: Option<String>,
    pub marketing_consent: bool,
    pub active: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomerInputJson {
    pub tenant_id: Option<i64>,
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub cpf: Option<String>,
    pub phone: Option<String>,
    pub marketing_consent: Option<bool>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CustomerPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    pub active: Option<bool>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl CustomerPageQuery {
    pub fn to_page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomerAddressJson {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: Option<i64>,
    pub customer_id: i64,
    pub label: Option<String>,
    pub recipient: String,
    pub cep: String,
    pub logradouro: String,
    pub numero: String,
    pub complemento: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub uf: String,
    pub is_default: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomerAddressInputJson {
    pub tenant_id: Option<i64>,
    pub customer_id: i64,
    pub label: Option<String>,
    pub recipient: String,
    pub cep: String,
    pub logradouro: String,
    pub numero: String,
    pub complemento: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub uf: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CustomerAddressPageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
    #[serde(alias = "customer_id")]
    pub customer_id: Option<i64>,
    pub uf: Option<String>,
    #[serde(alias = "tenant_id")]
    pub tenant_id: Option<i64>,
}

impl CustomerAddressPageQuery {
    pub fn to_page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }
}
