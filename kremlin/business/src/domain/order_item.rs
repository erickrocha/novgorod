use crate::commons::entity_mapper::EntityMapper;
use entity::order_item_entity::{Model, ActiveModel};
use sea_orm::{Set, NotSet, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderItem {
 pub id: i64,
 pub uuid: String,
 pub tenant_id: i64,
 pub order_id: i64,
 pub sku_id: i64,
 pub sku_code: String,
 pub product_name: String,
 pub attributes_desc: Option<String>,
 pub quantity: i32,
 pub unit_price_cents: i64,
 pub discount_cents: i64,
 pub ncm: String,
 pub cfop: Option<String>,
 pub csosn: Option<String>,
 pub icms_rate_bp: i32,
 pub tax_cents: i64,
 pub total_cents: i64,
 pub created_at: chrono::NaiveDateTime,
 pub created_by: Option<String>,
 pub updated_at: chrono::NaiveDateTime,
 pub updated_by: Option<String>,
}

pub struct OrderItemEntityMapper;
impl EntityMapper<OrderItem, Model, ActiveModel> for OrderItemEntityMapper {
 fn build_active_model(d: OrderItem) -> ActiveModel {
 ActiveModel {
 id: if d.id == 0 { NotSet } else { Set(d.id) },
 uuid: Set(uuid::Uuid::parse_str(&d.uuid).expect("persisted domain UUID")),
 tenant_id: Set(d.tenant_id),
 order_id: Set(d.order_id),
 sku_id: Set(d.sku_id),
 sku_code: Set(d.sku_code),
 product_name: Set(d.product_name),
 attributes_desc: Set(d.attributes_desc),
 quantity: Set(d.quantity),
 unit_price_cents: Set(d.unit_price_cents),
 discount_cents: Set(d.discount_cents),
 ncm: Set(d.ncm),
 cfop: Set(d.cfop),
 csosn: Set(d.csosn),
 icms_rate_bp: Set(d.icms_rate_bp),
 tax_cents: Set(d.tax_cents),
 total_cents: Set(d.total_cents),
 created_at: Set(d.created_at),
 created_by: Set(d.created_by),
 updated_at: Set(d.updated_at),
 updated_by: Set(d.updated_by),
 }
 }
 fn from_model(e: Model) -> OrderItem {
 OrderItem {
 id: e.id,
 uuid: e.uuid.to_string(),
 tenant_id: e.tenant_id,
 order_id: e.order_id,
 sku_id: e.sku_id,
 sku_code: e.sku_code,
 product_name: e.product_name,
 attributes_desc: e.attributes_desc,
 quantity: e.quantity,
 unit_price_cents: e.unit_price_cents,
 discount_cents: e.discount_cents,
 ncm: e.ncm,
 cfop: e.cfop,
 csosn: e.csosn,
 icms_rate_bp: e.icms_rate_bp,
 tax_cents: e.tax_cents,
 total_cents: e.total_cents,
 created_at: e.created_at,
 created_by: e.created_by,
 updated_at: e.updated_at,
 updated_by: e.updated_by,
 }
 }
 fn from_active_model(e: ActiveModel) -> OrderItem { Self::from_model(e.try_into_model().expect("complete persisted active model")) }
}
