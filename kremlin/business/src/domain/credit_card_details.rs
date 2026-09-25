use crate::commons::entity_mapper::EntityMapper;
use entity::credit_card_details_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditCardDetails {
    pub id: i64,
    pub uuid: String,
    pub payment_id: i64,
    pub cardholder_name: String,
    pub brand: String,
    pub last_four_digits: String,
    pub expiration_month: i32,
    pub expiration_year: i32,
    pub created_at: chrono::NaiveDateTime,
    pub created_by: Option<String>,
    pub updated_at: chrono::NaiveDateTime,
    pub updated_by: Option<String>,
}

pub struct CreditCardDetailsEntityMapper;
impl EntityMapper<CreditCardDetails, Model, ActiveModel> for CreditCardDetailsEntityMapper {
    fn build_active_model(d: CreditCardDetails) -> ActiveModel {
        ActiveModel {
            id: if d.id == 0 { NotSet } else { Set(d.id) },
            uuid: Set(uuid::Uuid::parse_str(&d.uuid).expect("persisted domain UUID")),
            payment_id: Set(d.payment_id),
            cardholder_name: Set(d.cardholder_name),
            brand: Set(d.brand),
            last_four_digits: Set(d.last_four_digits),
            expiration_month: Set(d.expiration_month),
            expiration_year: Set(d.expiration_year),
            created_at: Set(d.created_at),
            created_by: Set(d.created_by),
            updated_at: Set(d.updated_at),
            updated_by: Set(d.updated_by),
        }
    }
    fn from_model(e: Model) -> CreditCardDetails {
        CreditCardDetails {
            id: e.id,
            uuid: e.uuid.to_string(),
            payment_id: e.payment_id,
            cardholder_name: e.cardholder_name,
            brand: e.brand,
            last_four_digits: e.last_four_digits,
            expiration_month: e.expiration_month,
            expiration_year: e.expiration_year,
            created_at: e.created_at,
            created_by: e.created_by,
            updated_at: e.updated_at,
            updated_by: e.updated_by,
        }
    }
    fn from_active_model(e: ActiveModel) -> CreditCardDetails {
        Self::from_model(e.try_into_model().expect("complete persisted active model"))
    }
}
