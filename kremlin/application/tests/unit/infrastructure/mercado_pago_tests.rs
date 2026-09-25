use super::*;
use serde_json::json;

#[test]
fn parse_payment_approved_maps_to_captured_with_card_metadata() {
    let payload = json!({
        "id": 123456789,
        "status": "approved",
        "transaction_amount": 195.0,
        "currency_id": "BRL",
        "external_reference": "torg-55-uuid-key",
        "collector_id": 99999,
        "payment_type_id": "credit_card",
        "payment_method_id": "master",
        "card": {
            "last_four_digits": "5432",
            "expiration_month": 11,
            "expiration_year": 2029,
            "cardholder": {
                "name": "Maria Silva"
            }
        }
    });

    let result = MercadoPago::parse_payment_json(payload).expect("successful parse");
    assert_eq!(result.reference, "123456789");
    assert_eq!(result.status, ProviderStatus::Captured);
    assert_eq!(result.amount_cents, 19500);
    assert_eq!(result.currency, "BRL");
    assert_eq!(result.external_reference, "torg-55-uuid-key");
    assert_eq!(result.collector_id, 99999);
    assert_eq!(result.payment_type_id, "credit_card");

    let card = result.card.expect("card metadata present");
    assert_eq!(card.brand.as_deref(), Some("master"));
    assert_eq!(card.last_four_digits.as_deref(), Some("5432"));
    assert_eq!(card.expiration_month, Some(11));
    assert_eq!(card.expiration_year, Some(2029));
    assert_eq!(card.cardholder_name.as_deref(), Some("Maria Silva"));
}

#[test]
fn parse_payment_status_mappings() {
    let make_payload = |status: &str| json!({
        "id": "987654321",
        "status": status,
        "transaction_amount": 100.0,
        "currency_id": "BRL",
        "collector_id": 1,
        "payment_type_id": "credit_card",
    });

    let r_auth = MercadoPago::parse_payment_json(make_payload("authorized")).unwrap();
    assert_eq!(r_auth.status, ProviderStatus::Authorized);

    let r_rej = MercadoPago::parse_payment_json(make_payload("rejected")).unwrap();
    assert_eq!(r_rej.status, ProviderStatus::Failed);

    let r_canc = MercadoPago::parse_payment_json(make_payload("cancelled")).unwrap();
    assert_eq!(r_canc.status, ProviderStatus::Failed);

    let r_ref = MercadoPago::parse_payment_json(make_payload("refunded")).unwrap();
    assert_eq!(r_ref.status, ProviderStatus::Failed);

    let r_pend = MercadoPago::parse_payment_json(make_payload("in_process")).unwrap();
    assert_eq!(r_pend.status, ProviderStatus::Pending);
}

#[test]
fn parse_payment_rejects_invalid_amount() {
    let payload = json!({
        "id": 1,
        "status": "approved",
        "transaction_amount": -10.0,
        "currency_id": "BRL",
        "collector_id": 1,
        "payment_type_id": "credit_card"
    });
    assert!(MercadoPago::parse_payment_json(payload).is_err());
}
