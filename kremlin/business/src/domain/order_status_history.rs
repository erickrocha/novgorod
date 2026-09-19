use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::order_status_history_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderStatusHistory {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub order_id: i64,
    pub from_status: Option<String>,
    pub to_status: String,
    pub actor_type: String,
    pub actor_id: Option<i64>,
    pub note: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
}

pub struct OrderStatusHistoryEntityMapper {}

impl EntityMapper<OrderStatusHistory, Model, ActiveModel> for OrderStatusHistoryEntityMapper {
    fn build_active_model(d: OrderStatusHistory) -> ActiveModel {
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
            order_id: Set(d.order_id),
            from_status: Set(d.from_status),
            to_status: Set(d.to_status),
            actor_type: Set(d.actor_type),
            actor_id: Set(d.actor_id),
            note: Set(d.note),
            created_at: NotSet,
            created_by: match d.created_by {
                Some(cb) => Set(Some(cb)),
                None => NotSet,
            },
        }
    }

    fn from_model(e: Model) -> OrderStatusHistory {
        OrderStatusHistory {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            order_id: e.order_id,
            from_status: e.from_status,
            to_status: e.to_status,
            actor_type: e.actor_type,
            actor_id: e.actor_id,
            note: e.note,
            created_at: Some(e.created_at),
            created_by: e.created_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> OrderStatusHistory {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => OrderStatusHistory {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                order_id: e.order_id.take().unwrap_or_default(),
                from_status: e.from_status.take().flatten(),
                to_status: e.to_status.take().unwrap_or_default(),
                actor_type: e.actor_type.take().unwrap_or_default(),
                actor_id: e.actor_id.take().flatten(),
                note: e.note.take().flatten(),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
            },
        }
    }
}

