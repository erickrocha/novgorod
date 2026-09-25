use crate::commons::entity_mapper::EntityMapper;
use entity::purchase_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purchase {
    pub id: i64,
    pub uuid: String,
    pub customer_id: i64,
    pub customer_name: String,
    pub customer_tax_id: String,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub currency: String,
    pub status: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub shipping_cents: i64,
    pub tax_total_cents: i64,
    pub total_cents: i64,
    pub idempotency_key: String,
    pub request_hash: String,
    pub created_at: chrono::NaiveDateTime,
    pub created_by: Option<String>,
    pub updated_at: chrono::NaiveDateTime,
    pub updated_by: Option<String>,
}

pub struct PurchaseEntityMapper;
impl EntityMapper<Purchase, Model, ActiveModel> for PurchaseEntityMapper {
    fn build_active_model(d: Purchase) -> ActiveModel {
        ActiveModel {
            id: if d.id == 0 { NotSet } else { Set(d.id) },
            uuid: Set(uuid::Uuid::parse_str(&d.uuid).expect("persisted domain UUID")),
            customer_id: Set(d.customer_id),
            customer_name: Set(d.customer_name),
            customer_tax_id: Set(d.customer_tax_id),
            customer_email: Set(d.customer_email),
            customer_phone: Set(d.customer_phone),
            currency: Set(d.currency),
            status: Set(d.status),
            subtotal_cents: Set(d.subtotal_cents),
            discount_cents: Set(d.discount_cents),
            shipping_cents: Set(d.shipping_cents),
            tax_total_cents: Set(d.tax_total_cents),
            total_cents: Set(d.total_cents),
            idempotency_key: Set(d.idempotency_key),
            request_hash: Set(d.request_hash),
            created_at: Set(d.created_at),
            created_by: Set(d.created_by),
            updated_at: Set(d.updated_at),
            updated_by: Set(d.updated_by),
        }
    }
    fn from_model(e: Model) -> Purchase {
        Purchase {
            id: e.id,
            uuid: e.uuid.to_string(),
            customer_id: e.customer_id,
            customer_name: e.customer_name,
            customer_tax_id: e.customer_tax_id,
            customer_email: e.customer_email,
            customer_phone: e.customer_phone,
            currency: e.currency,
            status: e.status,
            subtotal_cents: e.subtotal_cents,
            discount_cents: e.discount_cents,
            shipping_cents: e.shipping_cents,
            tax_total_cents: e.tax_total_cents,
            total_cents: e.total_cents,
            idempotency_key: e.idempotency_key,
            request_hash: e.request_hash,
            created_at: e.created_at,
            created_by: e.created_by,
            updated_at: e.updated_at,
            updated_by: e.updated_by,
        }
    }
    fn from_active_model(e: ActiveModel) -> Purchase {
        Self::from_model(e.try_into_model().expect("complete persisted active model"))
    }
}
