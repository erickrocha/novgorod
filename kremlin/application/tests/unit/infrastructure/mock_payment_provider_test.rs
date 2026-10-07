use super::*;
use std::sync::Mutex;

static MOCK_PROVIDER_ENV_LOCK: Mutex<()> = Mutex::new(());

fn charge_request(provider_token: &str) -> ChargeRequest {
    ChargeRequest {
        payment_id: 42,
        amount_cents: 2500,
        currency: "BRL".into(),
        idempotency_key: "mock-test-key".into(),
        provider_token: provider_token.into(),
        external_reference: "torg-42-test-reference".into(),
        payer_email: "buyer@example.test".into(),
        payer_tax_id: "12345678901".into(),
        payment_method_id: "visa".into(),
        issuer_id: None,
        installments: 1,
    }
}

#[tokio::test]
async fn payload_outcomes_map_to_normalized_provider_results() {
    let outcomes = [
        ("mock:approved", ProviderStatus::Captured),
        ("mock:authorized", ProviderStatus::Authorized),
        ("mock:pending", ProviderStatus::Pending),
        ("mock:declined", ProviderStatus::Failed),
        ("mock:paid", ProviderStatus::Captured),
        ("mock:waiting", ProviderStatus::Pending),
    ];

    for provider_kind in [MockProviderKind::MercadoPago, MockProviderKind::PagSeguro] {
        let provider = MockPaymentProvider::new(provider_kind);
        for (payload, expected_status) in outcomes {
            let charge = provider.charge(charge_request(payload)).await.unwrap();
            assert_eq!(charge.status, expected_status, "{provider_kind:?} {payload}");
            assert_eq!(charge.amount_cents, 2500);
            assert_eq!(charge.currency, "BRL");
            assert_eq!(charge.external_reference, "torg-42-test-reference");
            assert_eq!(charge.payment_type_id, "credit_card");
            assert_eq!(charge.card.as_ref().unwrap().last_four_digits.as_deref(), Some("4242"));

            let status = provider.status(&charge.reference).await.unwrap();
            assert_eq!(status.status, expected_status, "status lookup for {payload}");
            assert_eq!(status.amount_cents, charge.amount_cents);
            assert_eq!(status.external_reference, charge.external_reference);
        }
    }
}

#[tokio::test]
async fn json_payload_selects_mock_outcome() {
    let provider = MockPaymentProvider::new(MockProviderKind::PagSeguro);
    let result = provider
        .charge(charge_request(r#"{"mockOutcome":"in_analysis"}"#))
        .await
        .unwrap();
    assert_eq!(result.status, ProviderStatus::Pending);
}

#[tokio::test]
async fn error_and_unrecognized_payloads_fail_closed() {
    let provider = MockPaymentProvider::new(MockProviderKind::MercadoPago);
    assert!(provider.charge(charge_request("mock:error")).await.is_err());
    assert!(provider.charge(charge_request("real-card-token")).await.is_err());
    assert!(provider.status("not-a-mock-reference").await.is_err());
}

#[tokio::test]
async fn search_does_not_invent_an_existing_charge() {
    let provider = MockPaymentProvider::new(MockProviderKind::MercadoPago);
    assert!(provider
        .search("torg-42-no-previous-charge")
        .await
        .unwrap()
        .is_none());
}

#[test]
fn configured_provider_requires_a_valid_provider_name() {
    let _guard = MOCK_PROVIDER_ENV_LOCK.lock().unwrap();
    let previous = std::env::var_os("PAYMENT_MOCK_PROVIDER");

    unsafe {
        std::env::set_var("PAYMENT_MOCK_PROVIDER", "pagseguro");
    }
    assert_eq!(MockPaymentProvider::configured().unwrap().name(), "pagseguro");

    unsafe {
        std::env::set_var("PAYMENT_MOCK_PROVIDER", "other");
    }
    assert!(MockPaymentProvider::configured().is_err());

    unsafe {
        if let Some(previous) = previous {
            std::env::set_var("PAYMENT_MOCK_PROVIDER", previous);
        } else {
            std::env::remove_var("PAYMENT_MOCK_PROVIDER");
        }
    }
}