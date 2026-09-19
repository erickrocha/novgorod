use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::coupon_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coupon {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub code: String,
    pub campaign_id: Option<i64>,
    pub coupon_type: String,
    pub value: i32,
    pub min_order_cents: Option<i32>,
    pub max_uses: Option<i32>,
    pub max_uses_per_customer: Option<i32>,
    pub starts_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
    pub active: bool,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct CouponEntityMapper {}

impl EntityMapper<Coupon, Model, ActiveModel> for CouponEntityMapper {
    fn build_active_model(d: Coupon) -> ActiveModel {
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
            code: Set(d.code),
            campaign_id: Set(d.campaign_id),
            coupon_type: Set(d.coupon_type),
            value: Set(d.value),
            min_order_cents: Set(d.min_order_cents),
            max_uses: Set(d.max_uses),
            max_uses_per_customer: Set(d.max_uses_per_customer),
            starts_at: Set(d.starts_at),
            expires_at: Set(d.expires_at),
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

    fn from_model(e: Model) -> Coupon {
        Coupon {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            code: e.code,
            campaign_id: e.campaign_id,
            coupon_type: e.coupon_type,
            value: e.value,
            min_order_cents: e.min_order_cents,
            max_uses: e.max_uses,
            max_uses_per_customer: e.max_uses_per_customer,
            starts_at: e.starts_at,
            expires_at: e.expires_at,
            active: e.active,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> Coupon {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => Coupon {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                code: e.code.take().unwrap_or_default(),
                campaign_id: e.campaign_id.take().flatten(),
                coupon_type: e.coupon_type.take().unwrap_or_default(),
                value: e.value.take().unwrap_or_default(),
                min_order_cents: e.min_order_cents.take().flatten(),
                max_uses: e.max_uses.take().flatten(),
                max_uses_per_customer: e.max_uses_per_customer.take().flatten(),
                starts_at: e.starts_at.take().flatten(),
                expires_at: e.expires_at.take().flatten(),
                active: e.active.take().unwrap_or(true),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
                updated_at: e.updated_at.take(),
                updated_by: e.updated_by.take().flatten(),
            },
        }
    }
}
