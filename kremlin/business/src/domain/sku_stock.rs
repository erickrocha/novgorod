use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::sku_stock_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

pub use entity::sku_stock_entity::{
    ActiveModel as SkuStockActiveModel, Model as SkuStockModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkuStock {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub sku_id: i64,
    pub quantity: i32,
    pub reserved: i32,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct SkuStockEntityMapper;

impl EntityMapper<SkuStock, Model, ActiveModel> for SkuStockEntityMapper {
    fn build_active_model(d: SkuStock) -> ActiveModel {
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
            sku_id: Set(d.sku_id),
            quantity: Set(d.quantity),
            reserved: Set(d.reserved),
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

    fn from_model(e: Model) -> SkuStock {
        SkuStock {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            sku_id: e.sku_id,
            quantity: e.quantity,
            reserved: e.reserved,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(e: ActiveModel) -> SkuStock {
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
    fn test_sku_stock_mapper_roundtrip() {
        let uuid = Uuid::new_v4();
        let now = Utc::now().naive_utc();
        let model = Model {
            id: 5,
            uuid,
            tenant_id: Some(1),
            sku_id: 100,
            quantity: 50,
            reserved: 5,
            created_at: now,
            created_by: Some("system".into()),
            updated_at: now,
            updated_by: None,
        };
        let domain = SkuStockEntityMapper::from_model(model);
        assert_eq!(domain.id, Some(5));
        assert_eq!(domain.uuid, Some(uuid_to_string(uuid)));
        assert_eq!(domain.sku_id, 100);
        assert_eq!(domain.quantity, 50);
        assert_eq!(domain.reserved, 5);

        let active = SkuStockEntityMapper::build_active_model(domain);
        assert_eq!(active.id.unwrap(), 5);
        assert_eq!(active.sku_id.unwrap(), 100);
        assert_eq!(active.quantity.unwrap(), 50);
        assert_eq!(active.reserved.unwrap(), 5);
    }
}
