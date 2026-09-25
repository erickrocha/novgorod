    use super::*;

    #[test]
    fn test_shipping_rate_mapper() {
        let now = chrono::Utc::now().naive_utc();
        let domain = business::domain::shipping_rate::ShippingRate {
            id: Some(1),
            uuid: Some("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(10),
            uf: "SP".to_string(),
            price_cents: 2500,
            created_at: Some(now),
            updated_at: Some(now),
            created_by: None,
            updated_by: None,
        };

        let json = ShippingRateMapper::json(domain.clone());
        assert_eq!(json.uf, "SP");
        assert_eq!(json.price_cents, 2500);
        assert_eq!(json.tenant_id, Some(10));

        let back_to_domain = ShippingRateMapper::domain(json);
        assert_eq!(back_to_domain.uf, domain.uf);
        assert_eq!(back_to_domain.price_cents, domain.price_cents);
    }

    #[test]
    fn test_customer_mapper() {
        let now = chrono::Utc::now().naive_utc();
        let domain = business::domain::customer::Customer {
            id: Some(42),
            uuid: Some("b1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(5),
            user_id: Some(10),
            name: "Alice Smith".to_string(),
            email: "alice@example.com".to_string(),
            cpf: Some("12345678901".to_string()),
            phone: Some("+5511999999999".to_string()),
            marketing_consent: true,
            consent_at: None,
            active: true,
            created_at: Some(now),
            updated_at: Some(now),
            created_by: None,
            updated_by: None,
        };

        let json = CustomerMapper::json(domain.clone());
        assert_eq!(json.name, "Alice Smith");
        assert_eq!(json.email, "alice@example.com");
        assert_eq!(json.cpf, Some("12345678901".to_string()));

        let back = CustomerMapper::domain(json);
        assert_eq!(back.name, domain.name);
        assert_eq!(back.email, domain.email);
        assert_eq!(back.cpf, domain.cpf);
    }

    #[test]
    fn test_coupon_mapper() {
        let now = chrono::Utc::now().naive_utc();
        let domain = business::domain::coupon::Coupon {
            id: Some(7),
            uuid: Some("c1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(1),
            campaign_id: Some(2),
            code: "SUMMER10".to_string(),
            coupon_type: "percentage".to_string(),
            value: 10,
            min_order_cents: None,
            max_uses: Some(100),
            max_uses_per_customer: Some(1),
            starts_at: None,
            expires_at: None,
            active: true,
            created_at: Some(now),
            updated_at: Some(now),
            created_by: None,
            updated_by: None,
        };

        let json = CouponMapper::json(domain.clone());
        assert_eq!(json.code, "SUMMER10");
        assert_eq!(json.coupon_type, "percentage");
        assert_eq!(json.value, 10);

        let back = CouponMapper::domain(json);
        assert_eq!(back.code, domain.code);
        assert_eq!(back.coupon_type, domain.coupon_type);
    }

    #[test]
    fn test_product_category_mapper() {
        let domain = ProductCategory {
            id: Some(15),
            uuid: Some("d1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(1),
            product_id: 100,
            category_id: 200,
            is_primary: true,
            created_at: None,
            created_by: None,
        };
        let json = ProductCategoryMapper::json(domain.clone());
        assert_eq!(json.id, 15);
        assert_eq!(json.product_id, 100);
        assert_eq!(json.category_id, 200);
        assert!(json.is_primary);

        let back = ProductCategoryMapper::domain(json);
        assert_eq!(back.id, domain.id);
        assert_eq!(back.product_id, domain.product_id);
    }

    #[test]
    fn test_sku_attribute_value_mapper() {
        let domain = SkuAttributeValue {
            id: Some(25),
            uuid: Some("e1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(1),
            product_id: 10,
            sku_id: 20,
            product_attribute_id: 30,
            attribute_id: 40,
            attribute_value_id: 50,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        };
        let json = SkuAttributeValueMapper::json(domain.clone());
        assert_eq!(json.sku_id, 20);
        assert_eq!(json.attribute_value_id, 50);

        let back = SkuAttributeValueMapper::domain(json);
        assert_eq!(back.sku_id, domain.sku_id);
        assert_eq!(back.attribute_value_id, domain.attribute_value_id);
    }

    #[test]
    fn test_sku_stock_mapper() {
        let domain = SkuStock {
            id: Some(35),
            uuid: Some("f1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(1),
            sku_id: 20,
            quantity: 150,
            reserved: 10,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        };
        let json = SkuStockMapper::json(domain.clone());
        assert_eq!(json.quantity, 150);
        assert_eq!(json.reserved, 10);

        let back = SkuStockMapper::domain(json);
        assert_eq!(back.quantity, domain.quantity);
        assert_eq!(back.reserved, domain.reserved);
    }

    #[test]
    fn test_person_mapper() {
        let dob = chrono::NaiveDate::from_ymd_opt(1995, 3, 20).unwrap();
        let domain = Person {
            id: Some(1),
            uuid: Some("abc-123".to_string()),
            tenant_id: Some(10),
            user_id: 100,
            first_name: "Jane".to_string(),
            surname: Some("Doe".to_string()),
            date_of_birth: Some(dob),
            gender: Some("F".to_string()),
            avatar: Some("jane.png".to_string()),
            phone: Some("+5511999999999".to_string()),
            email: Some("jane@example.com".to_string()),
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        };
        let json = PersonMapper::json(domain.clone());
        assert_eq!(json.id, 1);
        assert_eq!(json.first_name, "Jane");
        assert_eq!(json.surname, Some("Doe".to_string()));
        assert_eq!(json.date_of_birth, Some(dob));
        assert_eq!(json.gender, Some("F".to_string()));

        let back = PersonMapper::domain(json.clone());
        assert_eq!(back.id, domain.id);
        assert_eq!(back.first_name, domain.first_name);
        assert_eq!(back.surname, domain.surname);
        assert_eq!(back.date_of_birth, Some(dob));
        assert_eq!(json.avatar_url, None);

        let resource_profile = crate::endpoints::json::person_json::ResourceProfileJson {
            user: crate::endpoints::json::user_json::UserJson {
                id: Some(100),
                uuid: Some("user-uuid".to_string()),
                name: Some("Jane".to_string()),
                email: "jane@example.com".to_string(),
                password: None,
                enabled: true,
                first_login: false,
                role: "TenantUser".to_string(),
                tenant_id: Some(10),
                created_at: None,
                created_by: None,
                updated_at: None,
                updated_by: None,
            },
            person: Some(json),
            addresses: Vec::new(),
        };
        let serialized = serde_json::to_string(&resource_profile).unwrap();
        assert!(serialized.contains("\"user\":"));
        assert!(serialized.contains("\"person\":"));
        assert!(serialized.contains("\"addresses\":[]"));
        assert!(serialized.contains("\"avatarUrl\":null"));
    }

    #[test]
    fn test_person_address_mapper() {
        let domain = PersonAddress {
            id: Some(2),
            uuid: Some("def-456".to_string()),
            tenant_id: Some(10),
            person_id: 1,
            address_line1: Some("Av Paulista 1000".to_string()),
            address_line2: Some("Apto 101".to_string()),
            locality: Some("Bela Vista".to_string()),
            administrative_area: Some("SP".to_string()),
            postal_code: Some("01310-100".to_string()),
            country_code: Some("BR".to_string()),
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        };
        let json = PersonAddressMapper::json(domain.clone());
        assert_eq!(json.id.unwrap(), 2);
        assert_eq!(json.person_id, 1);
        assert_eq!(json.locality, Some("Bela Vista".to_string()));

        let back = PersonAddressMapper::domain(json);
        assert_eq!(back.id, domain.id);
        assert_eq!(back.person_id, domain.person_id);
        assert_eq!(back.locality, domain.locality);
    }
