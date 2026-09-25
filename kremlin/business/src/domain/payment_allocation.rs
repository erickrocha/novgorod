use crate::commons::entity_mapper::EntityMapper;
use entity::payment_allocation_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentAllocation {
    pub id: i64,
    pub uuid: String,
    pub purchase_id: i64,
    pub payment_id: i64,
    pub order_id: i64,
    pub amount_cents: i64,
    pub created_at: chrono::NaiveDateTime,
    pub created_by: Option<String>,
    pub updated_at: chrono::NaiveDateTime,
    pub updated_by: Option<String>,
}

pub struct PaymentAllocationEntityMapper;
impl EntityMapper<PaymentAllocation, Model, ActiveModel> for PaymentAllocationEntityMapper {
    fn build_active_model(d: PaymentAllocation) -> ActiveModel {
        ActiveModel {
            id: if d.id == 0 { NotSet } else { Set(d.id) },
            uuid: Set(uuid::Uuid::parse_str(&d.uuid).expect("persisted domain UUID")),
            purchase_id: Set(d.purchase_id),
            payment_id: Set(d.payment_id),
            order_id: Set(d.order_id),
            amount_cents: Set(d.amount_cents),
            created_at: Set(d.created_at),
            created_by: Set(d.created_by),
            updated_at: Set(d.updated_at),
            updated_by: Set(d.updated_by),
        }
    }
    fn from_model(e: Model) -> PaymentAllocation {
        PaymentAllocation {
            id: e.id,
            uuid: e.uuid.to_string(),
            purchase_id: e.purchase_id,
            payment_id: e.payment_id,
            order_id: e.order_id,
            amount_cents: e.amount_cents,
            created_at: e.created_at,
            created_by: e.created_by,
            updated_at: e.updated_at,
            updated_by: e.updated_by,
        }
    }
    fn from_active_model(e: ActiveModel) -> PaymentAllocation {
        Self::from_model(e.try_into_model().expect("complete persisted active model"))
    }
}
