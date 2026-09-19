use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::catalog_attribute_value_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

pub use entity::catalog_attribute_value_entity::{
    ActiveModel as CatalogAttributeValueActiveModel, Model as CatalogAttributeValueModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogAttributeValue {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub attribute_id: i64,
    pub value: String,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct CatalogAttributeValueEntityMapper;

impl EntityMapper<CatalogAttributeValue, Model, ActiveModel> for CatalogAttributeValueEntityMapper {
    fn build_active_model(d: CatalogAttributeValue) -> ActiveModel {
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
            attribute_id: Set(d.attribute_id),
            value: Set(d.value),
            created_at: match d.created_at {
                Some(dt) => Set(dt),
                None => NotSet,
            },
            created_by: Set(d.created_by),
            updated_at: match d.updated_at {
                Some(dt) => Set(dt),
                None => NotSet,
            },
            updated_by: Set(d.updated_by),
        }
    }

    fn from_model(e: Model) -> CatalogAttributeValue {
        CatalogAttributeValue {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            attribute_id: e.attribute_id,
            value: e.value,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(e: ActiveModel) -> CatalogAttributeValue {
        match e.try_into_model() {
            Ok(m) => Self::from_model(m),
            Err(_) => panic!("Failed to convert ActiveModel to Model"),
        }
    }
}
