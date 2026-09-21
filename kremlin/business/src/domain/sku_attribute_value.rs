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
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_sku_attribute_value_mapper_roundtrip() {
        let uuid = Uuid::new_v4();
        let now = Utc::now().naive_utc();
        let model = Model {
            id: 1,
            uuid,
            tenant_id: Some(2),
            product_id: 3,
            sku_id: 4,
            product_attribute_id: 5,
            attribute_id: 6,
            attribute_value_id: 7,
            created_at: now,
            created_by: Some("system".into()),
            updated_at: now,
            updated_by: None,
        };
        let domain = SkuAttributeValueEntityMapper::from_model(model);
        assert_eq!(domain.id, Some(1));
        assert_eq!(domain.uuid, Some(uuid_to_string(uuid)));
        assert_eq!(domain.sku_id, 4);
        assert_eq!(domain.attribute_id, 6);
        assert_eq!(domain.attribute_value_id, 7);

        let active = SkuAttributeValueEntityMapper::build_active_model(domain);
        assert_eq!(active.id.unwrap(), 1);
        assert_eq!(active.sku_id.unwrap(), 4);
        assert_eq!(active.attribute_value_id.unwrap(), 7);
    }
}
