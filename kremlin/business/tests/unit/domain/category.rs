    use super::*;
    use uuid::Uuid;

    #[test]
    fn from_model_converts_i8_and_uuid() {
        let uuid = Uuid::new_v4();
        let model = Model {
            id: 3,
            uuid: uuid.clone(),
            tenant_id: None,
            name: "Eletrônicos".into(),
            slug: "eletronicos".into(),
            parent_id: None,
            active: true,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None,
        };
        let domain = CategoryEntityMapper::from_model(model);
        assert!(domain.active);
        assert_eq!(domain.uuid.unwrap(), uuid_to_string(uuid));
    }
