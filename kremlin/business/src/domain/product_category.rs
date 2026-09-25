use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::product_category_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

pub use entity::product_category_entity::{
    ActiveModel as ProductCategoryActiveModel, Model as ProductCategoryModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductCategory {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub category_id: i64,
    pub is_primary: bool,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
}

pub struct ProductCategoryEntityMapper;

impl EntityMapper<ProductCategory, Model, ActiveModel> for ProductCategoryEntityMapper {
    fn build_active_model(d: ProductCategory) -> ActiveModel {
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
            category_id: Set(d.category_id),
            is_primary: Set(d.is_primary),
            created_at: match d.created_at {
                Some(dt) => Set(dt),
                None => NotSet,
            },
            created_by: Set(d.created_by),
        }
    }

    fn from_model(e: Model) -> ProductCategory {
        ProductCategory {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            product_id: e.product_id,
            category_id: e.category_id,
            is_primary: e.is_primary,
            created_at: Some(e.created_at),
            created_by: e.created_by,
        }
    }

    fn from_active_model(e: ActiveModel) -> ProductCategory {
        match e.try_into_model() {
            Ok(m) => Self::from_model(m),
            Err(_) => panic!("Failed to convert ActiveModel to Model"),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/domain/product_category.rs"]
mod tests;
