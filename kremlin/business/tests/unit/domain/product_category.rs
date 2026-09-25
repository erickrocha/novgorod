    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_product_category_mapper_roundtrip() {
        let uuid = Uuid::new_v4();
        let now = Utc::now().naive_utc();
        let model = Model {
            id: 10,
            uuid,
            tenant_id: Some(1),
            product_id: 20,
            category_id: 30,
            is_primary: true,
            created_at: now,
            created_by: Some("admin".into()),
        };
        let domain = ProductCategoryEntityMapper::from_model(model);
        assert_eq!(domain.id, Some(10));
        assert_eq!(domain.uuid, Some(uuid_to_string(uuid)));
        assert_eq!(domain.product_id, 20);
        assert_eq!(domain.category_id, 30);
        assert!(domain.is_primary);

        let active = ProductCategoryEntityMapper::build_active_model(domain);
        assert_eq!(active.id.unwrap(), 10);
        assert_eq!(active.product_id.unwrap(), 20);
        assert_eq!(active.category_id.unwrap(), 30);
        assert!(active.is_primary.unwrap());
    }
