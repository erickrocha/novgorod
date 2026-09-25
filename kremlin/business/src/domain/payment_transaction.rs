use crate::commons::entity_mapper::EntityMapper;
use entity::payment_transaction_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentTransaction {
    pub id: i64,
    pub uuid: String,
    pub payment_id: i64,
    pub operation: String,
    pub status: String,
    pub amount_cents: i64,
    pub currency: String,
    pub idempotency_key: String,
    pub gateway_provider: Option<String>,
    pub gateway_transaction_id: Option<String>,
    pub authorization_code: Option<String>,
    pub response_code: Option<String>,
    pub response_message: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub created_by: Option<String>,
}

pub struct PaymentTransactionEntityMapper;
impl EntityMapper<PaymentTransaction, Model, ActiveModel> for PaymentTransactionEntityMapper {
    fn build_active_model(d: PaymentTransaction) -> ActiveModel {
        ActiveModel {
            id: if d.id == 0 { NotSet } else { Set(d.id) },
            uuid: Set(uuid::Uuid::parse_str(&d.uuid).expect("persisted domain UUID")),
            payment_id: Set(d.payment_id),
            operation: Set(d.operation),
            status: Set(d.status),
            amount_cents: Set(d.amount_cents),
            currency: Set(d.currency),
            idempotency_key: Set(d.idempotency_key),
            gateway_provider: Set(d.gateway_provider),
            gateway_transaction_id: Set(d.gateway_transaction_id),
            authorization_code: Set(d.authorization_code),
            response_code: Set(d.response_code),
            response_message: Set(d.response_message),
            created_at: Set(d.created_at),
            created_by: Set(d.created_by),
        }
    }
    fn from_model(e: Model) -> PaymentTransaction {
        PaymentTransaction {
            id: e.id,
            uuid: e.uuid.to_string(),
            payment_id: e.payment_id,
            operation: e.operation,
            status: e.status,
            amount_cents: e.amount_cents,
            currency: e.currency,
            idempotency_key: e.idempotency_key,
            gateway_provider: e.gateway_provider,
            gateway_transaction_id: e.gateway_transaction_id,
            authorization_code: e.authorization_code,
            response_code: e.response_code,
            response_message: e.response_message,
            created_at: e.created_at,
            created_by: e.created_by,
        }
    }
    fn from_active_model(e: ActiveModel) -> PaymentTransaction {
        Self::from_model(e.try_into_model().expect("complete persisted active model"))
    }
}
