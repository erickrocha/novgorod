    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_sku_stock_mapper_roundtrip() {
        let uuid = Uuid::new_v4();
        let now = Utc::now().naive_utc();
        let model = Model {
            id: 5,
            uuid,
            tenant_id: Some(1),
            sku_id: 100,
            quantity: 50,
            reserved: 5,
            created_at: now,
            created_by: Some("system".into()),
            updated_at: now,
            updated_by: None,
        };
        let domain = SkuStockEntityMapper::from_model(model);
        assert_eq!(domain.id, Some(5));
        assert_eq!(domain.uuid, Some(uuid_to_string(uuid)));
        assert_eq!(domain.sku_id, 100);
        assert_eq!(domain.quantity, 50);
        assert_eq!(domain.reserved, 5);

        let active = SkuStockEntityMapper::build_active_model(domain);
        assert_eq!(active.id.unwrap(), 5);
        assert_eq!(active.sku_id.unwrap(), 100);
        assert_eq!(active.quantity.unwrap(), 50);
        assert_eq!(active.reserved.unwrap(), 5);
    }
