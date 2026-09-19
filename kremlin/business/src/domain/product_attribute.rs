use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::product_attribute_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

pub use entity::product_attribute_entity::{
    ActiveModel as ProductAttributeActiveModel, Model as ProductAttributeModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductAttribute {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub attribute_id: i64,
    pub required: bool,
    pub sort_order: i32,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
}

pub struct ProductAttributeEntityMapper;

impl EntityMapper<ProductAttribute, Model, ActiveModel> for ProductAttributeEntityMapper {
    fn build_active_model(d: ProductAttribute) -> ActiveModel {
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
            attribute_id: Set(d.attribute_id),
            required: Set(d.required),
            sort_order: Set(d.sort_order),
            created_at: match d.created_at {
                Some(dt) => Set(dt),
                None => NotSet,
            },
            created_by: Set(d.created_by),
        }
    }

    fn from_model(e: Model) -> ProductAttribute {
        ProductAttribute {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            product_id: e.product_id,
            attribute_id: e.attribute_id,
            required: e.required,
            sort_order: e.sort_order,
            created_at: Some(e.created_at),
            created_by: e.created_by,
        }
    }

    fn from_active_model(e: ActiveModel) -> ProductAttribute {
        match e.try_into_model() {
            Ok(m) => Self::from_model(m),
            Err(_) => panic!("Failed to convert ActiveModel to Model"),
        }
    }
}
