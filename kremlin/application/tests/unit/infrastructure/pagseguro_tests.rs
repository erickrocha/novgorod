use super::*;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde_json::json;
use std::sync::{Arc, Mutex};

async fn provider_error_response() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_GATEWAY,
        Json(json!({
            "error_messages": [{
                "code": "50099",
                "description": "token=token-marker tax_id=tax-marker card=card-marker"
            }]
        })),
    )
}

#[tokio::test]
async fn provider_error_bodies_are_not_exposed_by_charge_or_status() {
    let app = Router::new()
        .route("/charges", post(provider_error_response))
        .route("/charges/{reference}", get(provider_error_response));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let provider = PagSeguro::with_base_url_for_test(
        "tenant-token-marker".into(),
        format!("http://{address}"),
    )
    .unwrap();

    let charge_error = provider
        .charge(ChargeRequest {
            payment_id: 1,
            amount_cents: 5000,
            currency: "BRL".into(),
            idempotency_key: "attempt-marker".into(),
            provider_token: "card-token-marker".into(),
            external_reference: "purchase-marker".into(),
            payer_email: "buyer@example.test".into(),
            payer_tax_id: "tax-id-marker".into(),
            payment_method_id: "visa".into(),
            issuer_id: None,
            installments: 1,
        })
        .await
        .err()
        .expect("charge should return provider error")
        .0;
    let status_error = provider
        .status("CHAR_LOCAL_DOUBLE")
        .await
        .err()
        .expect("status should return provider error")
        .0;

    assert!(charge_error.contains("50099"));
    for error in [charge_error, status_error] {
        for marker in ["token-marker", "tax-marker", "card-marker", "tax-id-marker"] {
            assert!(!error.contains(marker), "{error}");
        }
    }
    server.abort();
}

async fn status_double(
    State(authorization): State<Arc<Mutex<Option<String>>>>,
    headers: HeaderMap,
) -> Json<serde_json::Value> {
    *authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    Json(json!({
        "id": "CHAR_LOCAL_DOUBLE",
        "reference_id": "torg-42-attempt",
        "status": "DECLINED",
        "amount": {"value": 25000, "currency": "BRL"}
    }))
}

#[tokio::test]
async fn status_sends_tenant_token_to_local_provider_double() {
    let authorization = Arc::new(Mutex::new(None));
    let app = Router::new()
        .route("/charges/{reference}", get(status_double))
        .with_state(authorization.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let provider = PagSeguro::with_base_url_for_test(
        "tenant-pagbank-token-sentinel".into(),
        format!("http://{address}"),
    )
    .unwrap();

    let result = provider.status("CHAR_LOCAL_DOUBLE").await.unwrap();

    assert_eq!(result.status, ProviderStatus::Failed);
    assert_eq!(
        authorization.lock().unwrap().as_deref(),
        Some("Bearer tenant-pagbank-token-sentinel")
    );
    server.abort();
}

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

#[tokio::test]
#[ignore]
async fn live_sandbox_pagseguro_charge_and_status() {
    let token = std::env::var("PAGSEGURO_TOKEN").expect("PAGSEGURO_TOKEN env variable required");
    let provider =
        PagSeguro::with_credentials(token, Some("sandbox")).expect("valid sandbox credentials");

    let card_json = json!({
        "number": "4539620659922097",
        "exp_month": "12",
        "exp_year": "2028",
        "security_code": "123",
        "holder": { "name": "PAGSEGURO TESTE" }
    })
    .to_string();

    let attempt_uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
    let external_ref = format!("torg-999-{attempt_uuid}");

    let result = provider
        .charge(ChargeRequest {
            payment_id: 999,
            amount_cents: 2500,
            currency: "BRL".into(),
            idempotency_key: format!("live-idemp-{}", &attempt_uuid[..8]),
            provider_token: card_json,
            external_reference: external_ref.clone(),
            payer_email: "erick.teste@novgorod.com".into(),
            payer_tax_id: "12345678909".into(),
            payment_method_id: "visa".into(),
            issuer_id: None,
            installments: 1,
        })
        .await
        .expect("charge against PagSeguro sandbox should succeed");

    println!("Live Sandbox Charge Success: ID={}", result.reference);
    assert_eq!(result.status, ProviderStatus::Captured);
    assert_eq!(result.amount_cents, 2500);
    assert_eq!(result.currency, "BRL");

    // Status / Re-fetch
    let fetched = provider
        .status(&result.reference)
        .await
        .expect("status lookup should succeed");
    println!("Live Sandbox Status Success: Status={:?}", fetched.status);
    assert_eq!(fetched.status, ProviderStatus::Captured);
    assert_eq!(fetched.reference, result.reference);
}

