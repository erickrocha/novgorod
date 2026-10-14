use super::*;
use serde_json::json;

async fn provider_error_response() -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    (
        axum::http::StatusCode::BAD_GATEWAY,
        axum::Json(json!({
            "message": "token=token-marker tax_id=tax-marker card=card-marker"
        })),
    )
}

#[tokio::test]
async fn provider_error_bodies_are_not_exposed_by_charge_or_status() {
    let app = axum::Router::new()
        .route("/v1/payments", axum::routing::post(provider_error_response))
        .route(
            "/v1/payments/{reference}",
            axum::routing::get(provider_error_response),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let provider = MercadoPago {
        client: reqwest::Client::new(),
        access_token: "tenant-token-marker".into(),
        collector_id: 7,
        base_url: format!("http://{address}"),
    };

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
        .status("123456789")
        .await
        .err()
        .expect("status should return provider error")
        .0;

    for error in [charge_error, status_error] {
        assert!(error.contains("502"));
        for marker in ["token-marker", "tax-marker", "card-marker", "tax-id-marker"] {
            assert!(!error.contains(marker), "{error}");
        }
    }
    server.abort();
}

/// Parses a provider payload the way the production search and charge paths do.
fn parse_payment_json(json: serde_json::Value) -> Result<ProviderResult, ProviderError> {
    let response: PaymentResponse =
        serde_json::from_value(json).map_err(|e| ProviderError(e.to_string()))?;
    MercadoPago::normalized_response(response)
}

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

    let result = parse_payment_json(payload).expect("successful parse");
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

    let r_auth = parse_payment_json(make_payload("authorized")).unwrap();
    assert_eq!(r_auth.status, ProviderStatus::Authorized);

    let r_rej = parse_payment_json(make_payload("rejected")).unwrap();
    assert_eq!(r_rej.status, ProviderStatus::Failed);

    let r_canc = parse_payment_json(make_payload("cancelled")).unwrap();
    assert_eq!(r_canc.status, ProviderStatus::Failed);

    let r_ref = parse_payment_json(make_payload("refunded")).unwrap();
    assert_eq!(r_ref.status, ProviderStatus::Failed);

    let r_pend = parse_payment_json(make_payload("in_process")).unwrap();
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
    assert!(parse_payment_json(payload).is_err());
}

struct SandboxTestCard {
    number: String,
    expiration_month: u32,
    expiration_year: u32,
    security_code: String,
}

fn load_sandbox_test_card(path: &str, brand: &str) -> Result<SandboxTestCard, &'static str> {
    let contents = std::fs::read_to_string(path).map_err(|_| "cannot read Sandbox test cards file")?;
    let section = format!("{brand}:");
    let mut in_brand = false;
    let mut number = None;
    let mut expiry = None;
    let mut security_code = None;

    for line in contents.lines() {
        let trimmed = line.trim();
        if line.starts_with("  ") && !line.starts_with("    ") && trimmed.ends_with(':') {
            in_brand = trimmed == section;
        } else if in_brand {
            if let Some(value) = trimmed.strip_prefix("number:") {
                number = Some(
                    value
                        .chars()
                        .filter(|character| !character.is_whitespace())
                        .collect::<String>(),
                );
            } else if let Some(value) = trimmed.strip_prefix("expiry:") {
                expiry = Some(value.trim().to_owned());
            } else if let Some(value) = trimmed.strip_prefix("cvv:") {
                security_code = Some(value.trim().to_owned());
            }
        }
    }

    let (month, year) = expiry
        .as_deref()
        .and_then(|value| value.split_once('/'))
        .ok_or("Sandbox card expiry must use MM/YY format")?;
    let month = month
        .parse::<u32>()
        .map_err(|_| "Sandbox card expiry month must be numeric")?;
    let mut year = year
        .parse::<u32>()
        .map_err(|_| "Sandbox card expiry year must be numeric")?;
    if year < 100 {
        year += 2000;
    }

    Ok(SandboxTestCard {
        number: number
            .filter(|value| !value.is_empty())
            .ok_or("Sandbox test card number is missing")?,
        expiration_month: month,
        expiration_year: year,
        security_code: security_code
            .filter(|value| !value.is_empty())
            .ok_or("Sandbox test card security code is missing")?,
    })
}

#[test]
fn loads_visa_sandbox_card_without_exposing_card_data() {
    let cards_file = std::env::var("MP_SANDBOX_TEST_CARDS_FILE")
        .expect("MP_SANDBOX_TEST_CARDS_FILE is required");
    let card = load_sandbox_test_card(&cards_file, "visa").unwrap();
    assert_eq!(card.number.len(), 16);
    assert_eq!(card.expiration_month, 11);
    assert_eq!(card.expiration_year, 2030);
    assert_eq!(card.security_code.len(), 3);
}

#[tokio::test]
#[ignore = "creates a real test payment with the Mercado Pago Sandbox"]
async fn live_sandbox_mercado_pago_charge_and_status() {
    let access_token = std::env::var("MP_SANDBOX_ACCESS_TOKEN")
        .expect("MP_SANDBOX_ACCESS_TOKEN is required");
    let public_key = std::env::var("MP_SANDBOX_PUBLIC_KEY")
        .expect("MP_SANDBOX_PUBLIC_KEY is required");
    let buyer_email = std::env::var("MP_SANDBOX_BUYER_EMAIL")
        .expect("MP_SANDBOX_BUYER_EMAIL must be the email of a Mercado Pago test buyer");
    let collector_id = std::env::var("MP_SANDBOX_COLLECTOR_ID")
        .expect("MP_SANDBOX_COLLECTOR_ID is required")
        .parse::<i64>()
        .expect("MP_SANDBOX_COLLECTOR_ID must be numeric");
    let provider = MercadoPago::with_credentials(access_token, Some(collector_id))
        .expect("valid Mercado Pago Sandbox credentials");
    let cards_file = std::env::var("MP_SANDBOX_TEST_CARDS_FILE")
        .expect("MP_SANDBOX_TEST_CARDS_FILE is required");
    let client = reqwest::Client::new();

    for (cardholder_name, expected_status) in [
        ("APRO", ProviderStatus::Captured),
        ("OTHE", ProviderStatus::Failed),
        ("CONT", ProviderStatus::Pending),
    ] {
        let card = load_sandbox_test_card(&cards_file, "visa")
            .expect("Visa test card must be configured in cards.yaml");
        let token_response = client
            .post("https://api.mercadopago.com/v1/card_tokens")
            .query(&[("public_key", public_key.as_str())])
            .json(&json!({
                "card_number": card.number,
                "expiration_month": card.expiration_month,
                "expiration_year": card.expiration_year,
                "security_code": card.security_code,
                "cardholder": {
                    "name": cardholder_name,
                    "identification": {
                        "type": "CPF",
                        "number": "12345678909"
                    }
                }
            }))
            .send()
            .await
            .expect("Mercado Pago Sandbox card-token request should complete");
        let token_status = token_response.status();
        assert!(
            token_status.is_success(),
            "Mercado Pago Sandbox card-token request returned {token_status}"
        );
        let token_body: serde_json::Value = token_response
            .json()
            .await
            .expect("Mercado Pago Sandbox card-token response should be JSON");
        let card_token = token_body
            .get("id")
            .and_then(serde_json::Value::as_str)
            .filter(|id| !id.is_empty())
            .expect("Mercado Pago Sandbox card-token response should contain an id")
            .to_owned();

        let attempt_id = uuid::Uuid::new_v4().to_string();
        let external_reference = format!("novgorod-mp-sandbox-{attempt_id}");
        let result = provider
            .charge(ChargeRequest {
                payment_id: 1,
                amount_cents: 100,
                currency: "BRL".into(),
                idempotency_key: attempt_id,
                provider_token: card_token,
                external_reference,
                payer_email: buyer_email.clone(),
                payer_tax_id: "12345678909".into(),
                payment_method_id: "visa".into(),
                issuer_id: None,
                installments: 1,
            })
            .await
            .expect("Mercado Pago Sandbox payment should return a result");

        assert_eq!(result.status, expected_status, "{cardholder_name} status");
        assert_eq!(result.amount_cents, 100);
        assert_eq!(result.currency, "BRL");
        assert_eq!(result.collector_id, collector_id);

        let fetched = provider
            .status(&result.reference)
            .await
            .expect("Mercado Pago Sandbox status lookup should succeed");
        assert_eq!(fetched.status, expected_status, "{cardholder_name} re-fetched status");
        assert_eq!(fetched.reference, result.reference);
        assert_eq!(fetched.amount_cents, result.amount_cents);
        assert_eq!(fetched.collector_id, collector_id);
    }
    println!("Mercado Pago Sandbox APRO, OTHE, and CONT scenarios passed.");
}
