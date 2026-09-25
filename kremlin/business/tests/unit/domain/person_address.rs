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
