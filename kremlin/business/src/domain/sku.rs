use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::sku_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

pub use entity::sku_entity::{ActiveModel as SkuActiveModel, Model as SkuModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sku {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub product_id: i64,
    pub code: String,
    pub variant_key: String,
    pub price_cents: i32,
    pub compare_at_price_cents: Option<i32>,
    pub weight_g: Option<i32>,
    pub width_mm: Option<i32>,
    pub height_mm: Option<i32>,
    pub length_mm: Option<i32>,
    pub active: bool,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct SkuEntityMapper;

impl EntityMapper<Sku, Model, ActiveModel> for SkuEntityMapper {
    fn build_active_model(d: Sku) -> ActiveModel {
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
            code: Set(d.code),
            variant_key: Set(d.variant_key),
            price_cents: Set(d.price_cents),
            compare_at_price_cents: Set(d.compare_at_price_cents),
            weight_g: Set(d.weight_g),
            width_mm: Set(d.width_mm),
            height_mm: Set(d.height_mm),
            length_mm: Set(d.length_mm),
            active: Set(d.active),
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

    fn from_model(e: Model) -> Sku {
        Sku {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            product_id: e.product_id,
            code: e.code,
            variant_key: e.variant_key,
            price_cents: e.price_cents,
            compare_at_price_cents: e.compare_at_price_cents,
            weight_g: e.weight_g,
            width_mm: e.width_mm,
            height_mm: e.height_mm,
            length_mm: e.length_mm,
            active: e.active,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(e: ActiveModel) -> Sku {
        match e.try_into_model() {
            Ok(m) => Self::from_model(m),
            Err(_) => panic!("Failed to convert ActiveModel to Model"),
        }
    }
}
