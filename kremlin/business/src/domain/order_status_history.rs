use crate::commons::entity_mapper::EntityMapper;
use entity::order_status_history_entity::{Model, ActiveModel};
use sea_orm::{Set, NotSet, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderStatusHistory {
 pub id: i64,
 pub uuid: String,
 pub tenant_id: i64,
 pub order_id: i64,
 pub from_status: Option<String>,
 pub to_status: String,
 pub actor_type: String,
 pub actor_id: Option<i64>,
 pub note: Option<String>,
 pub created_at: chrono::NaiveDateTime,
 pub created_by: Option<String>,
}

pub struct OrderStatusHistoryEntityMapper;
impl EntityMapper<OrderStatusHistory, Model, ActiveModel> for OrderStatusHistoryEntityMapper {
 fn build_active_model(d: OrderStatusHistory) -> ActiveModel {
 ActiveModel {
 id: if d.id == 0 { NotSet } else { Set(d.id) },
 uuid: Set(uuid::Uuid::parse_str(&d.uuid).expect("persisted domain UUID")),
 tenant_id: Set(d.tenant_id),
 order_id: Set(d.order_id),
 from_status: Set(d.from_status),
 to_status: Set(d.to_status),
 actor_type: Set(d.actor_type),
 actor_id: Set(d.actor_id),
 note: Set(d.note),
 created_at: Set(d.created_at),
 created_by: Set(d.created_by),
 }
 }
 fn from_model(e: Model) -> OrderStatusHistory {
 OrderStatusHistory {
 id: e.id,
 uuid: e.uuid.to_string(),
 tenant_id: e.tenant_id,
 order_id: e.order_id,
 from_status: e.from_status,
 to_status: e.to_status,
 actor_type: e.actor_type,
 actor_id: e.actor_id,
 note: e.note,
 created_at: e.created_at,
 created_by: e.created_by,
 }
 }
 fn from_active_model(e: ActiveModel) -> OrderStatusHistory { Self::from_model(e.try_into_model().expect("complete persisted active model")) }
}
