use crate::commons::entity_mapper::EntityMapper;
use entity::payment_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payment {
    pub id: i64,
    pub uuid: String,
    pub purchase_id: i64,
    pub method: String,
    pub status: String,
    pub installments: i32,
    pub amount_cents: i64,
    pub currency: String,
    pub gateway_provider: Option<String>,
    pub gateway_reference: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub created_by: Option<String>,
    pub updated_at: chrono::NaiveDateTime,
    pub updated_by: Option<String>,
}

pub struct PaymentEntityMapper;
impl EntityMapper<Payment, Model, ActiveModel> for PaymentEntityMapper {
    fn build_active_model(d: Payment) -> ActiveModel {
        ActiveModel {
            id: if d.id == 0 { NotSet } else { Set(d.id) },
            uuid: Set(uuid::Uuid::parse_str(&d.uuid).expect("persisted domain UUID")),
            purchase_id: Set(d.purchase_id),
            method: Set(d.method),
            status: Set(d.status),
            installments: Set(d.installments),
            amount_cents: Set(d.amount_cents),
            currency: Set(d.currency),
            gateway_provider: Set(d.gateway_provider),
            gateway_reference: Set(d.gateway_reference),
            created_at: Set(d.created_at),
            created_by: Set(d.created_by),
            updated_at: Set(d.updated_at),
            updated_by: Set(d.updated_by),
        }
    }
    fn from_model(e: Model) -> Payment {
        Payment {
            id: e.id,
            uuid: e.uuid.to_string(),
            purchase_id: e.purchase_id,
            method: e.method,
            status: e.status,
            installments: e.installments,
            amount_cents: e.amount_cents,
            currency: e.currency,
            gateway_provider: e.gateway_provider,
            gateway_reference: e.gateway_reference,
            created_at: e.created_at,
            created_by: e.created_by,
            updated_at: e.updated_at,
            updated_by: e.updated_by,
        }
    }
    fn from_active_model(e: ActiveModel) -> Payment {
        Self::from_model(e.try_into_model().expect("complete persisted active model"))
    }
}
