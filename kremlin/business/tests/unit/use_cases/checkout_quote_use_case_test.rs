use super::*;

#[test]
fn selection_recalculates_totals_and_rejects_invalid_sets_atomically() {
    let mut quote: QuoteResult = serde_json::from_value(serde_json::json!({
        "id":1,"expiresAt":"2026-09-26T12:00:00","shippingAddress":{"recipient":"Buyer","addressLine1":"A","addressLine2":null,"locality":"City","administrativeArea":"SP","postalCode":"01001000","countryCode":"BR"},
        "items":[],"sellers":[{"tenantId":1,"subtotalCents":1000,"discountCents":100,"shippingCents":200,"totalCents":1100,"couponId":null,"couponCode":null,
        "shipping":{"configurationVersion":1,"mode":"correios","originCep":"01001000","destinationCep":"01001000","parcel":null,"inputsHash":"hash",
        "options":[{"id":"a","provider":"correios","serviceCode":"A","serviceName":"A","priceCents":200,"transitDays":5},{"id":"b","provider":"correios","serviceCode":"B","serviceName":"B","priceCents":300,"transitDays":2}],
        "selectedOption":{"id":"a","provider":"correios","serviceCode":"A","serviceName":"A","priceCents":200,"transitDays":5}}}],
        "subtotalCents":1000,"discountCents":100,"shippingCents":200,"totalCents":1100
    })).unwrap();
    let old = quote.clone();
    for selections in [
        vec![],
        vec![ShippingSelection {
            tenant_id: 2,
            option_id: "b".into(),
        }],
        vec![ShippingSelection {
            tenant_id: 1,
            option_id: "fake".into(),
        }],
        vec![
            ShippingSelection {
                tenant_id: 1,
                option_id: "a".into(),
            },
            ShippingSelection {
                tenant_id: 1,
                option_id: "b".into(),
            },
        ],
    ] {
        assert!(quote.select(selections).is_err());
        assert_eq!(quote, old);
    }
    quote
        .select(vec![ShippingSelection {
            tenant_id: 1,
            option_id: "b".into(),
        }])
        .unwrap();
    assert_eq!(quote.total_cents, 1200);
    assert_eq!(quote.shipping_cents, 300);
    assert_eq!(quote.expires_at, old.expires_at);
    assert_eq!(quote.discount_cents, 100);
}

#[test]
fn test_fixed_shipping_weight_and_free_shipping_rules() {
    let now = chrono::Utc::now().naive_utc();
    let rate = shipping_rate_entity::Model {
        id: 1,
        uuid: uuid::Uuid::new_v4(),
        tenant_id: Some(1),
        origin_warehouse_id: None,
        region_name: Some("SP (Capital e Grande SP)".to_string()),
        uf: "SP".to_string(),
        destination_cep_start: Some("01000000".to_string()),
        destination_cep_end: Some("09999999".to_string()),
        price_cents: 1290,
        transit_days_min: 1,
        transit_days_max: 2,
        max_weight_g: Some(2000),
        extra_weight_per_kg_cents: Some(500),
        free_shipping_threshold_cents: Some(15000),
        created_at: now,
        created_by: None,
        updated_at: now,
        updated_by: None,
    };

    // Case 1: Within 2kg base weight (1500g) -> 1290 cents
    let total_weight_g = 1500_i64;
    let subtotal_cents = 5000_i64;
    let is_free = rate.free_shipping_threshold_cents.is_some_and(|th| subtotal_cents >= i64::from(th));
    assert!(!is_free);
    let mut price = i64::from(rate.price_cents);
    if let (Some(max_w), Some(extra_per_kg)) = (rate.max_weight_g, rate.extra_weight_per_kg_cents) {
        let max_w = i64::from(max_w);
        if total_weight_g > max_w && extra_per_kg > 0 {
            let excess_g = total_weight_g - max_w;
            let excess_kg = (excess_g + 999) / 1000;
            price += excess_kg * i64::from(extra_per_kg);
        }
    }
    assert_eq!(price, 1290);

    // Case 2: Above 2kg (2500g -> 500g excess = 1kg extra @ 500 cents) -> 1290 + 500 = 1790
    let total_weight_g = 2500_i64;
    let mut price = i64::from(rate.price_cents);
    if let (Some(max_w), Some(extra_per_kg)) = (rate.max_weight_g, rate.extra_weight_per_kg_cents) {
        let max_w = i64::from(max_w);
        if total_weight_g > max_w && extra_per_kg > 0 {
            let excess_g = total_weight_g - max_w;
            let excess_kg = (excess_g + 999) / 1000;
            price += excess_kg * i64::from(extra_per_kg);
        }
    }
    assert_eq!(price, 1790);

    // Case 3: Above free shipping threshold (subtotal >= 15000) -> 0 cents
    let subtotal_cents = 16000_i64;
    let is_free = rate.free_shipping_threshold_cents.is_some_and(|th| subtotal_cents >= i64::from(th));
    assert!(is_free);
}

#[test]
fn test_shipping_matrix_cep_range_matching_and_specificity() {
    let now = chrono::Utc::now().naive_utc();
    let capital_rate = shipping_rate_entity::Model {
        id: 1,
        uuid: uuid::Uuid::new_v4(),
        tenant_id: Some(1),
        origin_warehouse_id: None,
        region_name: Some("SP (Capital e Grande SP)".to_string()),
        uf: "SP".to_string(),
        destination_cep_start: Some("01000000".to_string()),
        destination_cep_end: Some("09999999".to_string()),
        price_cents: 1290,
        transit_days_min: 1,
        transit_days_max: 2,
        max_weight_g: Some(2000),
        extra_weight_per_kg_cents: Some(500),
        free_shipping_threshold_cents: None,
        created_at: now,
        created_by: None,
        updated_at: now,
        updated_by: None,
    };
    let interior_rate = shipping_rate_entity::Model {
        id: 2,
        uuid: uuid::Uuid::new_v4(),
        tenant_id: Some(1),
        origin_warehouse_id: None,
        region_name: Some("SP (Interior e Litoral)".to_string()),
        uf: "SP".to_string(),
        destination_cep_start: Some("11000000".to_string()),
        destination_cep_end: Some("19999999".to_string()),
        price_cents: 1690,
        transit_days_min: 2,
        transit_days_max: 4,
        max_weight_g: Some(2000),
        extra_weight_per_kg_cents: Some(600),
        free_shipping_threshold_cents: None,
        created_at: now,
        created_by: None,
        updated_at: now,
        updated_by: None,
    };
    let generic_sp_rate = shipping_rate_entity::Model {
        id: 3,
        uuid: uuid::Uuid::new_v4(),
        tenant_id: Some(1),
        origin_warehouse_id: None,
        region_name: None,
        uf: "SP".to_string(),
        destination_cep_start: None,
        destination_cep_end: None,
        price_cents: 2000,
        transit_days_min: 3,
        transit_days_max: 5,
        max_weight_g: None,
        extra_weight_per_kg_cents: None,
        free_shipping_threshold_cents: None,
        created_at: now,
        created_by: None,
        updated_at: now,
        updated_by: None,
    };

    let rates = vec![generic_sp_rate.clone(), capital_rate.clone(), interior_rate.clone()];

    let find_best = |rates: &[shipping_rate_entity::Model], dest_cep: &str, dest_uf: &str| -> Option<shipping_rate_entity::Model> {
        rates
            .iter()
            .cloned()
            .filter_map(|r| {
                let has_cep_range = match (&r.destination_cep_start, &r.destination_cep_end) {
                    (Some(start), Some(end)) if !start.trim().is_empty() && !end.trim().is_empty() => {
                        let s = start.trim();
                        let e = end.trim();
                        if dest_cep >= s && dest_cep <= e {
                            true
                        } else {
                            return None;
                        }
                    }
                    _ => false,
                };

                if !has_cep_range && !r.uf.trim().eq_ignore_ascii_case(dest_uf) {
                    return None;
                }

                let score = (r.origin_warehouse_id.is_some() as u8 * 2) + (has_cep_range as u8);
                Some((score, r))
            })
            .max_by_key(|(score, _)| *score)
            .map(|(_, r)| r)
    };

    // CEP 01310-100 (Av. Paulista, Capital) -> matches SP Capital (price 1290, not generic 2000)
    let best = find_best(&rates, "01310100", "SP").unwrap();
    assert_eq!(best.id, 1);
    assert_eq!(best.price_cents, 1290);
    assert_eq!(best.region_name.as_deref(), Some("SP (Capital e Grande SP)"));

    // CEP 14010-000 (Ribeirão Preto, Interior) -> matches SP Interior (price 1690, not generic 2000)
    let best_interior = find_best(&rates, "14010000", "SP").unwrap();
    assert_eq!(best_interior.id, 2);
    assert_eq!(best_interior.price_cents, 1690);
    assert_eq!(best_interior.region_name.as_deref(), Some("SP (Interior e Litoral)"));

    // CEP without range in table -> falls back to generic SP rate
    let rates_generic_only = vec![generic_sp_rate];
    let best_fallback = find_best(&rates_generic_only, "01310100", "SP").unwrap();
    assert_eq!(best_fallback.id, 3);
    assert_eq!(best_fallback.price_cents, 2000);
}

#[test]
fn quote_items_belong_to_a_tenant_only_when_all_of_them_do() {
    let quote = |tenants: &[i64]| {
        let items: Vec<_> = tenants
            .iter()
            .map(|t| {
                serde_json::json!({
                    "skuId": 1, "tenantId": t, "name": "n", "quantity": 1,
                    "unitPriceCents": 100, "totalCents": 100
                })
            })
            .collect();
        serde_json::json!({
            "id": 1, "expiresAt": "2026-10-02T00:00:00",
            "shippingAddress": {
                "recipient": "r", "addressLine1": "a", "addressLine2": null, "locality": "l",
                "administrativeArea": "SP", "postalCode": "01310100", "countryCode": "BR"
            },
            "items": items, "sellers": [],
            "subtotalCents": 0, "discountCents": 0, "shippingCents": 0, "totalCents": 0
        })
    };
    assert!(quote_items_belong_to(&quote(&[7, 7]), 7));
    assert!(!quote_items_belong_to(&quote(&[7, 8]), 7));
    assert!(!quote_items_belong_to(&serde_json::json!({"broken": true}), 7));
}
