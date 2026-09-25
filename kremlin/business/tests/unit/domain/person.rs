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
