use crate::commons::entity_mapper::EntityMapper;
use crate::commons::functions::{string_to_uuid, uuid_to_string};
use chrono::{NaiveDate, NaiveDateTime};
use entity::person_entity::{ActiveModel, Model};
use sea_orm::{NotSet, Set, TryIntoModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub tenant_id: Option<i64>,
    pub user_id: i64,
    pub first_name: String,
    pub surname: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

pub struct PersonEntityMapper {}

impl EntityMapper<Person, Model, ActiveModel> for PersonEntityMapper {
    fn build_active_model(d: Person) -> ActiveModel {
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
            user_id: Set(d.user_id),
            first_name: Set(d.first_name),
            surname: Set(d.surname),
            date_of_birth: Set(d.date_of_birth),
            gender: Set(d.gender),
            avatar: Set(d.avatar),
            phone: Set(d.phone),
            email: Set(d.email),
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

    fn from_model(e: Model) -> Person {
        Person {
            id: Some(e.id),
            uuid: Some(uuid_to_string(e.uuid)),
            tenant_id: e.tenant_id,
            user_id: e.user_id,
            first_name: e.first_name,
            surname: e.surname,
            date_of_birth: e.date_of_birth,
            gender: e.gender,
            avatar: e.avatar,
            phone: e.phone,
            email: e.email,
            created_at: Some(e.created_at),
            created_by: e.created_by,
            updated_at: Some(e.updated_at),
            updated_by: e.updated_by,
        }
    }

    fn from_active_model(mut e: ActiveModel) -> Person {
        let model: Result<Model, _> = e.clone().try_into_model();
        match model {
            Ok(m) => Self::from_model(m),
            Err(_) => Person {
                id: e.id.take(),
                uuid: e.uuid.take().map(uuid_to_string),
                tenant_id: e.tenant_id.take().flatten(),
                user_id: e.user_id.take().unwrap_or_default(),
                first_name: e.first_name.take().unwrap_or_default(),
                surname: e.surname.take().flatten(),
                date_of_birth: e.date_of_birth.take().flatten(),
                gender: e.gender.take().flatten(),
                avatar: e.avatar.take().flatten(),
                phone: e.phone.take().flatten(),
                email: e.email.take().flatten(),
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
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    #[test]
    fn test_person_mapper_roundtrip() {
        let uuid = Uuid::new_v4();
        let now = Utc::now().naive_utc();
        let dob = NaiveDate::from_ymd_opt(1990, 5, 15).unwrap();
        let model = Model {
            id: 1,
            uuid,
            tenant_id: Some(10),
            user_id: 20,
            first_name: "John".into(),
            surname: Some("Doe".into()),
            date_of_birth: Some(dob),
            gender: Some("M".into()),
            avatar: Some("avatar.png".into()),
            phone: Some("+123456789".into()),
            email: Some("john.doe@example.com".into()),
            created_at: now,
            created_by: Some("admin".into()),
            updated_at: now,
            updated_by: None,
        };

        let domain = PersonEntityMapper::from_model(model);
        assert_eq!(domain.id, Some(1));
        assert_eq!(domain.uuid, Some(uuid_to_string(uuid)));
        assert_eq!(domain.tenant_id, Some(10));
        assert_eq!(domain.user_id, 20);
        assert_eq!(domain.first_name, "John");
        assert_eq!(domain.surname, Some("Doe".into()));
        assert_eq!(domain.date_of_birth, Some(dob));
        assert_eq!(domain.gender, Some("M".into()));
        assert_eq!(domain.avatar, Some("avatar.png".into()));
        assert_eq!(domain.phone, Some("+123456789".into()));
        assert_eq!(domain.email, Some("john.doe@example.com".into()));

        let active_model = PersonEntityMapper::build_active_model(domain);
        let converted = PersonEntityMapper::from_active_model(active_model);
        assert_eq!(converted.id, Some(1));
        assert_eq!(converted.first_name, "John");
        assert_eq!(converted.surname, Some("Doe".into()));
        assert_eq!(converted.date_of_birth, Some(dob));
        assert_eq!(converted.gender, Some("M".into()));

        // Also test with empty / None fields
        let empty_person = Person {
            id: None,
            uuid: None,
            tenant_id: Some(5),
            user_id: 15,
            first_name: "Alice".into(),
            surname: None,
            date_of_birth: None,
            gender: None,
            avatar: None,
            phone: None,
            email: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        };
        let am = PersonEntityMapper::build_active_model(empty_person);
        let back = PersonEntityMapper::from_active_model(am);
        assert_eq!(back.first_name, "Alice");
        assert_eq!(back.surname, None);
        assert_eq!(back.date_of_birth, None);
        assert_eq!(back.gender, None);
        assert_eq!(back.email, None);
    }
}
