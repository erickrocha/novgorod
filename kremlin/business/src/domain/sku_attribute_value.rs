use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::sku_attribute_value_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

pub use entity::sku_attribute_value_entity::{
    ActiveModel as SkuAttributeValueActiveModel, Model as SkuAttributeValueModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkuAttributeValue {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub sku_id: i64,
    pub product_attribute_id: i64,
    pub attribute_id: i64,
    pub attribute_value_id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub type SkuAttribute = SkuAttributeValue;

pub struct SkuAttributeValueEntityMapper;
pub type SkuAttributeEntityMapper = SkuAttributeValueEntityMapper;

impl EntityMapper<SkuAttributeValue, Model, ActiveModel> for SkuAttributeValueEntityMapper {
    fn build_active_model(d: SkuAttributeValue) -> ActiveModel {
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
            product_id: Set(d.product_id),
            sku_id: Set(d.sku_id),
            product_attribute_id: Set(d.product_attribute_id),
            attribute_id: Set(d.attribute_id),
            attribute_value_id: Set(d.attribute_value_id),
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

    fn from_model(e: Model) -> SkuAttributeValue {
        SkuAttributeValue {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            product_id: e.product_id,
            sku_id: e.sku_id,
            product_attribute_id: e.product_attribute_id,
            attribute_id: e.attribute_id,
            attribute_value_id: e.attribute_value_id,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(e: ActiveModel) -> SkuAttributeValue {
        match e.try_into_model() {
            Ok(m) => Self::from_model(m),
            Err(_) => panic!("Failed to convert ActiveModel to Model"),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/domain/sku_attribute_value.rs"]
mod tests;
