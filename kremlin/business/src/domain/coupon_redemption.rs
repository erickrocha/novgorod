use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::coupon_redemption_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouponRedemption {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub coupon_id: i64,
    pub order_id: i64,
    pub customer_id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
}

pub struct CouponRedemptionEntityMapper {}

impl EntityMapper<CouponRedemption, Model, ActiveModel> for CouponRedemptionEntityMapper {
    fn build_active_model(d: CouponRedemption) -> ActiveModel {
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
            coupon_id: Set(d.coupon_id),
            order_id: Set(d.order_id),
            customer_id: Set(d.customer_id),
            created_at: NotSet,
            created_by: match d.created_by {
                Some(cb) => Set(Some(cb)),
                None => NotSet,
            },
        }
    }

    fn from_model(e: Model) -> CouponRedemption {
        CouponRedemption {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            coupon_id: e.coupon_id,
            order_id: e.order_id,
            customer_id: e.customer_id,
            created_at: Some(e.created_at),
            created_by: e.created_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> CouponRedemption {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => CouponRedemption {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                coupon_id: e.coupon_id.take().unwrap_or_default(),
                order_id: e.order_id.take().unwrap_or_default(),
                customer_id: e.customer_id.take().unwrap_or_default(),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
            },
        }
    }
}

