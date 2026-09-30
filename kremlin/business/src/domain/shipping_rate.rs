use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::shipping_rate_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShippingRate {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub origin_warehouse_id: Option<i64>,
    pub region_name: Option<String>,
    pub uf: String,
    pub destination_cep_start: Option<String>,
    pub destination_cep_end: Option<String>,
    pub price_cents: i32,
    pub transit_days_min: i32,
    pub transit_days_max: i32,
    pub max_weight_g: Option<i32>,
    pub extra_weight_per_kg_cents: Option<i32>,
    pub free_shipping_threshold_cents: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct ShippingRateEntityMapper {}

impl EntityMapper<ShippingRate, Model, ActiveModel> for ShippingRateEntityMapper {
    fn build_active_model(d: ShippingRate) -> ActiveModel {
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
            origin_warehouse_id: Set(d.origin_warehouse_id),
            region_name: Set(d.region_name),
            uf: Set(d.uf),
            destination_cep_start: Set(d.destination_cep_start),
            destination_cep_end: Set(d.destination_cep_end),
            price_cents: Set(d.price_cents),
            transit_days_min: Set(d.transit_days_min),
            transit_days_max: Set(d.transit_days_max),
            max_weight_g: Set(d.max_weight_g),
            extra_weight_per_kg_cents: Set(d.extra_weight_per_kg_cents),
            free_shipping_threshold_cents: Set(d.free_shipping_threshold_cents),
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

    fn from_model(e: Model) -> ShippingRate {
        ShippingRate {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            origin_warehouse_id: e.origin_warehouse_id,
            region_name: e.region_name,
            uf: e.uf,
            destination_cep_start: e.destination_cep_start,
            destination_cep_end: e.destination_cep_end,
            price_cents: e.price_cents,
            transit_days_min: e.transit_days_min,
            transit_days_max: e.transit_days_max,
            max_weight_g: e.max_weight_g,
            extra_weight_per_kg_cents: e.extra_weight_per_kg_cents,
            free_shipping_threshold_cents: e.free_shipping_threshold_cents,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> ShippingRate {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => ShippingRate {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                origin_warehouse_id: e.origin_warehouse_id.take().flatten(),
                region_name: e.region_name.take().flatten(),
                uf: e.uf.take().unwrap_or_default(),
                destination_cep_start: e.destination_cep_start.take().flatten(),
                destination_cep_end: e.destination_cep_end.take().flatten(),
                price_cents: e.price_cents.take().unwrap_or_default(),
                transit_days_min: e.transit_days_min.take().unwrap_or(1),
                transit_days_max: e.transit_days_max.take().unwrap_or(2),
                max_weight_g: e.max_weight_g.take().flatten(),
                extra_weight_per_kg_cents: e.extra_weight_per_kg_cents.take().flatten(),
                free_shipping_threshold_cents: e.free_shipping_threshold_cents.take().flatten(),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
                updated_at: e.updated_at.take(),
                updated_by: e.updated_by.take().flatten(),
            },
        }
    }
}

