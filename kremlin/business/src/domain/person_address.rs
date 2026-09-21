use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::NaiveDateTime;
use entity::person_address_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonAddress {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub person_id: i64,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub locality: Option<String>,
    pub administrative_area: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct PersonAddressEntityMapper {}

impl EntityMapper<PersonAddress, Model, ActiveModel> for PersonAddressEntityMapper {
    fn build_active_model(d: PersonAddress) -> ActiveModel {
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
            person_id: Set(d.person_id),
            address_line1: Set(d.address_line1),
            address_line2: Set(d.address_line2),
            locality: Set(d.locality),
            administrative_area: Set(d.administrative_area),
            postal_code: Set(d.postal_code),
            country_code: Set(d.country_code),
            created_at: NotSet,
            created_by: match d.created_by {
                Some(cb) => Set(Some(cb)),
                None => NotSet,
            },
            updated_at: NotSet,
            updated_by: match d.updated_by {
                Some(ub) => Set(Some(ub)),
                None => NotSet,
            },
        }
    }

    fn from_model(e: Model) -> PersonAddress {
        PersonAddress {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            person_id: e.person_id,
            address_line1: e.address_line1,
            address_line2: e.address_line2,
            locality: e.locality,
            administrative_area: e.administrative_area,
            postal_code: e.postal_code,
            country_code: e.country_code,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> PersonAddress {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => PersonAddress {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                person_id: e.person_id.take().unwrap_or_default(),
                address_line1: e.address_line1.take().flatten(),
                address_line2: e.address_line2.take().flatten(),
                locality: e.locality.take().flatten(),
                administrative_area: e.administrative_area.take().flatten(),
                postal_code: e.postal_code.take().flatten(),
                country_code: e.country_code.take().flatten(),
                created_at: e.created_at.take(),
                created_by: e.created_by.take().flatten(),
                updated_at: e.updated_at.take(),
                updated_by: e.updated_by.take().flatten(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_person_address_mapper_roundtrip() {
        let uuid = Uuid::new_v4();
        let now = Utc::now().naive_utc();
        let model = Model {
            id: 1,
            uuid,
            tenant_id: Some(10),
            person_id: 100,
            address_line1: Some("123 Main St".into()),
            address_line2: Some("Apt 4B".into()),
            locality: Some("Springfield".into()),
            administrative_area: Some("IL".into()),
            postal_code: Some("62701".into()),
            country_code: Some("US".into()),
            created_at: now,
            created_by: Some("admin".into()),
            updated_at: now,
            updated_by: None,
        };

        let domain = PersonAddressEntityMapper::from_model(model);
        assert_eq!(domain.id, Some(1));
        assert_eq!(domain.uuid, Some(uuid_to_string(uuid)));
        assert_eq!(domain.tenant_id, Some(10));
        assert_eq!(domain.person_id, 100);
        assert_eq!(domain.address_line1, Some("123 Main St".into()));
        assert_eq!(domain.address_line2, Some("Apt 4B".into()));
        assert_eq!(domain.locality, Some("Springfield".into()));
        assert_eq!(domain.administrative_area, Some("IL".into()));
        assert_eq!(domain.postal_code, Some("62701".into()));
        assert_eq!(domain.country_code, Some("US".into()));

        let active_model = PersonAddressEntityMapper::build_active_model(domain);
        let converted = PersonAddressEntityMapper::from_active_model(active_model);
        assert_eq!(converted.id, Some(1));
        assert_eq!(converted.person_id, 100);
        assert_eq!(converted.locality, Some("Springfield".into()));
    }
}
