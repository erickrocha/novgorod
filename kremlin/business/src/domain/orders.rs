use crate::commons::entity_mapper::EntityMapper;
use entity::orders_entity::{Model, ActiveModel};
use sea_orm::{Set, NotSet, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Orders {
 pub id: i64,
 pub uuid: String,
 pub purchase_id: i64,
 pub tenant_id: i64,
 pub number: String,
 pub customer_id: i64,
 pub customer_name: String,
 pub customer_tax_id: String,
 pub customer_email: Option<String>,
 pub customer_phone: Option<String>,
 pub status: String,
 pub payment_status: String,
 pub subtotal_cents: i64,
 pub discount_cents: i64,
 pub shipping_cents: i64,
 pub tax_total_cents: i64,
 pub total_cents: i64,
 pub coupon_id: Option<i64>,
 pub coupon_code: Option<String>,
 pub placed_at: chrono::NaiveDateTime,
 pub created_at: chrono::NaiveDateTime,
 pub created_by: Option<String>,
 pub updated_at: chrono::NaiveDateTime,
 pub updated_by: Option<String>,
}

pub struct OrdersEntityMapper;
impl EntityMapper<Orders, Model, ActiveModel> for OrdersEntityMapper {
 fn build_active_model(d: Orders) -> ActiveModel {
 ActiveModel {
 id: if d.id == 0 { NotSet } else { Set(d.id) },
 uuid: Set(uuid::Uuid::parse_str(&d.uuid).expect("persisted domain UUID")),
 purchase_id: Set(d.purchase_id),
 tenant_id: Set(d.tenant_id),
 number: Set(d.number),
 customer_id: Set(d.customer_id),
 customer_name: Set(d.customer_name),
 customer_tax_id: Set(d.customer_tax_id),
 customer_email: Set(d.customer_email),
 customer_phone: Set(d.customer_phone),
 status: Set(d.status),
 payment_status: Set(d.payment_status),
 subtotal_cents: Set(d.subtotal_cents),
 discount_cents: Set(d.discount_cents),
 shipping_cents: Set(d.shipping_cents),
 tax_total_cents: Set(d.tax_total_cents),
 total_cents: Set(d.total_cents),
 coupon_id: Set(d.coupon_id),
 coupon_code: Set(d.coupon_code),
 placed_at: Set(d.placed_at),
 created_at: Set(d.created_at),
 created_by: Set(d.created_by),
 updated_at: Set(d.updated_at),
 updated_by: Set(d.updated_by),
 }
 }
 fn from_model(e: Model) -> Orders {
 Orders {
 id: e.id,
 uuid: e.uuid.to_string(),
 purchase_id: e.purchase_id,
 tenant_id: e.tenant_id,
 number: e.number,
 customer_id: e.customer_id,
 customer_name: e.customer_name,
 customer_tax_id: e.customer_tax_id,
 customer_email: e.customer_email,
 customer_phone: e.customer_phone,
 status: e.status,
 payment_status: e.payment_status,
 subtotal_cents: e.subtotal_cents,
 discount_cents: e.discount_cents,
 shipping_cents: e.shipping_cents,
 tax_total_cents: e.tax_total_cents,
 total_cents: e.total_cents,
 coupon_id: e.coupon_id,
 coupon_code: e.coupon_code,
 placed_at: e.placed_at,
 created_at: e.created_at,
 created_by: e.created_by,
 updated_at: e.updated_at,
 updated_by: e.updated_by,
 }
 }
 fn from_active_model(e: ActiveModel) -> Orders { Self::from_model(e.try_into_model().expect("complete persisted active model")) }
}
