use super::*;
use serde_json::json;

#[test]
fn parse_payment_paid_maps_to_captured_with_card_metadata() {
    let payload = json!({
        "id": "CHAR_1A2B3C4D5E",
        "reference_id": "torg-42-uuid-test",
        "status": "PAID",
        "amount": {
            "value": 25000,
            "currency": "BRL"
        },
        "payment_method": {
            "type": "CREDIT_CARD",
            "installments": 2,
            "card": {
                "brand": "visa",
                "first_digits": "411111",
                "last_digits": "1234",
                "exp_month": 12,
                "exp_year": 2032,
                "holder": {
                    "name": "Carlos Souza"
                }
            }
        }
    });

    let result = PagSeguro::parse_payment_json(payload).expect("successful parse");
    assert_eq!(result.reference, "CHAR_1A2B3C4D5E");
    assert_eq!(result.status, ProviderStatus::Captured);
    assert_eq!(result.amount_cents, 25000);
    assert_eq!(result.currency, "BRL");
    assert_eq!(result.external_reference, "torg-42-uuid-test");
    assert_eq!(result.collector_id, 0);
    assert_eq!(result.payment_type_id, "credit_card");

    let card = result.card.expect("card metadata present");
    assert_eq!(card.brand.as_deref(), Some("visa"));
    assert_eq!(card.last_four_digits.as_deref(), Some("1234"));
    assert_eq!(card.expiration_month, Some(12));
    assert_eq!(card.expiration_year, Some(2032));
    assert_eq!(card.cardholder_name.as_deref(), Some("Carlos Souza"));
}

#[test]
fn parse_payment_status_mappings() {
    let make_payload = |status: &str| {
        json!({
            "id": "CHAR_STATUS_TEST",
            "status": status,
            "amount": {
                "value": 5000,
                "currency": "BRL"
            }
        })
    };

    let r_paid = PagSeguro::parse_payment_json(make_payload("PAID")).unwrap();
    assert_eq!(r_paid.status, ProviderStatus::Captured);

    let r_auth = PagSeguro::parse_payment_json(make_payload("AUTHORIZED")).unwrap();
    assert_eq!(r_auth.status, ProviderStatus::Authorized);

    let r_declined = PagSeguro::parse_payment_json(make_payload("DECLINED")).unwrap();
    assert_eq!(r_declined.status, ProviderStatus::Failed);

    let r_canceled = PagSeguro::parse_payment_json(make_payload("CANCELED")).unwrap();
    assert_eq!(r_canceled.status, ProviderStatus::Failed);

    let r_waiting = PagSeguro::parse_payment_json(make_payload("WAITING")).unwrap();
    assert_eq!(r_waiting.status, ProviderStatus::Pending);

    let r_analysis = PagSeguro::parse_payment_json(make_payload("IN_ANALYSIS")).unwrap();
    assert_eq!(r_analysis.status, ProviderStatus::Pending);
}

#[test]
fn parse_payment_rejects_negative_or_missing_amount() {
    let payload_negative = json!({
        "id": "CHAR_NEG",
        "status": "PAID",
        "amount": {
            "value": -100,
            "currency": "BRL"
        }
    });
    assert!(PagSeguro::parse_payment_json(payload_negative).is_err());

    let payload_missing_amount = json!({
        "id": "CHAR_NO_AMOUNT",
        "status": "PAID"
    });
    assert!(PagSeguro::parse_payment_json(payload_missing_amount).is_err());
}

#[test]
fn parse_payment_rejects_empty_id() {
    let payload = json!({
        "id": "",
        "status": "PAID",
        "amount": {
            "value": 1000,
            "currency": "BRL"
        }
    });
    assert!(PagSeguro::parse_payment_json(payload).is_err());
}

#[test]
fn test_with_credentials_validation() {
    assert!(PagSeguro::with_credentials("".into(), None).is_err());

    let ps_sandbox =
        PagSeguro::with_credentials("sample_token".into(), Some("sandbox")).unwrap();
    assert_eq!(ps_sandbox.base_url, "https://sandbox.api.pagseguro.com");

    let ps_prod =
        PagSeguro::with_credentials("sample_token".into(), Some("production")).unwrap();
    assert_eq!(ps_prod.base_url, "https://api.pagseguro.com");
}
