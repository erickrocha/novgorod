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
