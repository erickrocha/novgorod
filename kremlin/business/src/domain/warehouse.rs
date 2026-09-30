use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::warehouse_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warehouse {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub name: String,
    pub origin_cep: String,
    pub street: Option<String>,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub district: Option<String>,
    pub city: String,
    pub uf: String,
    pub is_default: bool,
    pub active: bool,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct WarehouseEntityMapper {}

impl EntityMapper<Warehouse, Model, ActiveModel> for WarehouseEntityMapper {
    fn build_active_model(d: Warehouse) -> ActiveModel {
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
            name: Set(d.name),
            origin_cep: Set(d.origin_cep),
            street: Set(d.street),
            number: Set(d.number),
            complement: Set(d.complement),
            district: Set(d.district),
            city: Set(d.city),
            uf: Set(d.uf),
            is_default: Set(d.is_default),
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

    fn from_model(e: Model) -> Warehouse {
        Warehouse {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            name: e.name,
            origin_cep: e.origin_cep,
            street: e.street,
            number: e.number,
            complement: e.complement,
            district: e.district,
            city: e.city,
            uf: e.uf,
            is_default: e.is_default,
            active: e.active,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> Warehouse {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => Warehouse {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                name: e.name.take().unwrap_or_default(),
                origin_cep: e.origin_cep.take().unwrap_or_default(),
                street: e.street.take().flatten(),
                number: e.number.take().flatten(),
                complement: e.complement.take().flatten(),
                district: e.district.take().flatten(),
                city: e.city.take().unwrap_or_default(),
                uf: e.uf.take().unwrap_or_default(),
                is_default: e.is_default.take().unwrap_or(false),
                active: e.active.take().unwrap_or(true),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
                updated_at: e.updated_at.take(),
                updated_by: e.updated_by.take().flatten(),
            },
        }
    }
}
