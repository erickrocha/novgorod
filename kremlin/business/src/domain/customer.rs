use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::customer_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Customer {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub user_id: Option<i64>,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub cpf: Option<String>,
    pub phone: Option<String>,
    pub marketing_consent: bool,
    pub consent_at: Option<NaiveDateTime>,
    pub active: bool,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct CustomerEntityMapper {}

impl EntityMapper<Customer, Model, ActiveModel> for CustomerEntityMapper {
    fn build_active_model(d: Customer) -> ActiveModel {
        ActiveModel {
            id: match d.id {
                Some(id) => Set(id),
                None => NotSet,
            },
            uuid: match d.uuid {
                Some(uuid) => Set(string_to_uuid(&uuid)),
                None => NotSet,
            },
            tenant_id: Set(d.tenant_id),
            user_id: Set(d.user_id),
            name: Set(d.name),
            email: Set(d.email),
            password_hash: Set(d.password_hash),
            cpf: Set(d.cpf),
            phone: Set(d.phone),
            marketing_consent: Set(d.marketing_consent),
            consent_at: Set(d.consent_at),
            active: Set(d.active),
            created_at: NotSet,
            created_by: match d.created_by {
                Some(cb) => Set(Some(cb)),
                None => NotSet,
            },
            updated_at: NotSet,
            updated_by: match d.updated_by {
                Some(ub) => Set(Some(ub)),
                None => NotSet,
            },
        }
    }

    fn from_model(e: Model) -> Customer {
        Customer {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            user_id: e.user_id,
            name: e.name,
            email: e.email,
            password_hash: e.password_hash,
            cpf: e.cpf,
            phone: e.phone,
            marketing_consent: e.marketing_consent,
            consent_at: e.consent_at,
            active: e.active,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> Customer {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => Customer {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                user_id: e.user_id.take().flatten(),
                name: e.name.take().unwrap_or_default(),
                email: e.email.take().unwrap_or_default(),
                password_hash: e.password_hash.take().unwrap_or_default(),
                cpf: e.cpf.take().flatten(),
                phone: e.phone.take().flatten(),
                marketing_consent: e.marketing_consent.take().unwrap_or_default(),
                consent_at: e.consent_at.take().flatten(),
                active: e.active.take().unwrap_or(true),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
                updated_at: e.updated_at.take(),
                updated_by: e.updated_by.take().flatten(),
            },
        }
    }
}

