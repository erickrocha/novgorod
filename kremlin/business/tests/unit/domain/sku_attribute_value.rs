    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_sku_attribute_value_mapper_roundtrip() {
        let uuid = Uuid::new_v4();
        let now = Utc::now().naive_utc();
        let model = Model {
            id: 1,
            uuid,
            tenant_id: Some(2),
            product_id: 3,
            sku_id: 4,
            product_attribute_id: 5,
            attribute_id: 6,
            attribute_value_id: 7,
            created_at: now,
            created_by: Some("system".into()),
            updated_at: now,
            updated_by: None,
        };
        let domain = SkuAttributeValueEntityMapper::from_model(model);
        assert_eq!(domain.id, Some(1));
        assert_eq!(domain.uuid, Some(uuid_to_string(uuid)));
        assert_eq!(domain.sku_id, 4);
        assert_eq!(domain.attribute_id, 6);
        assert_eq!(domain.attribute_value_id, 7);

        let active = SkuAttributeValueEntityMapper::build_active_model(domain);
        assert_eq!(active.id.unwrap(), 1);
        assert_eq!(active.sku_id.unwrap(), 4);
        assert_eq!(active.attribute_value_id.unwrap(), 7);
    }
