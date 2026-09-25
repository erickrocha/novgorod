use crate::commons::entity_mapper::EntityMapper;
use entity::order_address_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderAddress {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: i64,
    pub order_id: i64,
    pub address_type: String,
    pub recipient: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub locality: String,
    pub administrative_area: String,
    pub postal_code: String,
    pub country_code: String,
    pub created_at: chrono::NaiveDateTime,
    pub created_by: Option<String>,
    pub updated_at: chrono::NaiveDateTime,
    pub updated_by: Option<String>,
}

pub struct OrderAddressEntityMapper;
impl EntityMapper<OrderAddress, Model, ActiveModel> for OrderAddressEntityMapper {
    fn build_active_model(d: OrderAddress) -> ActiveModel {
        ActiveModel {
            id: if d.id == 0 { NotSet } else { Set(d.id) },
            uuid: Set(uuid::Uuid::parse_str(&d.uuid).expect("persisted domain UUID")),
            tenant_id: Set(d.tenant_id),
            order_id: Set(d.order_id),
            address_type: Set(d.address_type),
            recipient: Set(d.recipient),
            address_line1: Set(d.address_line1),
            address_line2: Set(d.address_line2),
            locality: Set(d.locality),
            administrative_area: Set(d.administrative_area),
            postal_code: Set(d.postal_code),
            country_code: Set(d.country_code),
            created_at: Set(d.created_at),
            created_by: Set(d.created_by),
            updated_at: Set(d.updated_at),
            updated_by: Set(d.updated_by),
        }
    }
    fn from_model(e: Model) -> OrderAddress {
        OrderAddress {
            id: e.id,
            uuid: e.uuid.to_string(),
            tenant_id: e.tenant_id,
            order_id: e.order_id,
            address_type: e.address_type,
            recipient: e.recipient,
            address_line1: e.address_line1,
            address_line2: e.address_line2,
            locality: e.locality,
            administrative_area: e.administrative_area,
            postal_code: e.postal_code,
            country_code: e.country_code,
            created_at: e.created_at,
            created_by: e.created_by,
            updated_at: e.updated_at,
            updated_by: e.updated_by,
        }
    }
    fn from_active_model(e: ActiveModel) -> OrderAddress {
        Self::from_model(e.try_into_model().expect("complete persisted active model"))
    }
}
