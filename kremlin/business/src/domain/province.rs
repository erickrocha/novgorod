use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use entity::province_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Province {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub acronym: String,
    pub name: String,
    pub country_code: String,
    pub ibge_code: Option<String>,
}

pub struct ProvinceEntityMapper {}

impl EntityMapper<Province, Model, ActiveModel> for ProvinceEntityMapper {
    fn build_active_model(d: Province) -> ActiveModel {
        ActiveModel {
            id: match d.id {
                Some(id) => Set(id),
                None => NotSet,
            },
            uuid: match d.uuid {
                Some(uuid) => Set(string_to_uuid(&uuid)),
                None => NotSet,
            },
            acronym: Set(d.acronym),
            name: Set(d.name),
            country_code: Set(d.country_code),
            ibge_code: match d.ibge_code {
                Some(code) => Set(Some(code)),
                None => NotSet,
            },
        }
    }

    fn from_model(e: Model) -> Province {
        Province {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            acronym: e.acronym,
            name: e.name,
            country_code: e.country_code,
            ibge_code: e.ibge_code,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> Province {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => Province {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                acronym: e.acronym.take().unwrap_or_default(),
                name: e.name.take().unwrap_or_default(),
                country_code: e.country_code.take().unwrap_or_default(),
                ibge_code: e.ibge_code.take().flatten(),
            },
        }
    }
}
