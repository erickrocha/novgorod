use super::*;

#[test]
fn mock_provider_mode_is_opt_in_and_validates_provider() {
    assert!(mock_provider_for_config(None, None).unwrap().is_none());
    assert!(mock_provider_for_config(Some("real"), Some("invalid")).unwrap().is_none());
    assert!(mock_provider_for_config(Some("unknown"), None).is_err());
    assert!(mock_provider_for_config(Some("mock"), None).is_err());
    assert!(mock_provider_for_config(Some("mock"), Some("invalid")).is_err());

    assert!(mock_provider_for_config(Some("mock"), Some("mercado_pago")).is_err());
    assert_eq!(
        mock_provider_for_config(Some("mock"), Some("pagseguro"))
            .unwrap()
            .unwrap()
            .name(),
        "pagseguro"
    );
}

fn ps_payload(token: &str, environment: Option<&str>) -> TenantCredentialsPayload {
    TenantCredentialsPayload::PagSeguro(crate::infrastructure::payment_credentials::PagSeguroCredentials {
        token: token.into(),
        public_key: None,
        environment: environment.map(str::to_owned),
    })
}

// NOV-5 TC-15 (SR-PAY-015): tenant credentials only, no silent fallback.
#[test]
fn tenant_provider_is_built_from_tenant_credentials_only() {
    let ps = build_tenant_provider("pagseguro", ps_payload("tok", Some("sandbox"))).unwrap();
    assert_eq!(ps.name(), "pagseguro");
    assert!(build_tenant_provider("pagseguro", ps_payload("tok", Some("production"))).is_ok());
}

#[test]
fn tenant_provider_rejects_column_mismatch_and_invalid_credentials() {
    assert!(build_tenant_provider("mercado_pago", ps_payload("tok", Some("sandbox"))).is_err());
    assert!(build_tenant_provider("pagseguro", ps_payload("", Some("sandbox"))).is_err());
}

#[test]
fn pagseguro_environment_is_validated_not_defaulted_to_sandbox() {
    assert!(build_tenant_provider("pagseguro", ps_payload("tok", None)).is_err());
    assert!(build_tenant_provider("pagseguro", ps_payload("tok", Some("prod"))).is_err());
    assert!(build_tenant_provider("pagseguro", ps_payload("tok", Some(""))).is_err());
}

// NOV-5 TC-19 (SR-PAY-017): the notification only names the payment; its status is never used.
#[test]
fn pagseguro_notification_yields_only_the_reference() {
    let body = br#"{"id":"CHAR_1","reference_id":"torg-5-key","status":"PAID","amount":{"value":1,"currency":"BRL"}}"#;
    assert_eq!(pagseguro_notification_reference(body), Some("torg-5-key".into()));
    let nested = br#"{"charges":[{"reference_id":"torg-6-key2","status":"PAID"}]}"#;
    assert_eq!(pagseguro_notification_reference(nested), Some("torg-6-key2".into()));
    assert_eq!(pagseguro_notification_reference(b"{}"), None);
    assert_eq!(pagseguro_notification_reference(b"garbage"), None);
}

#[test]
fn pagseguro_signature_uses_account_token_and_raw_request_body() {
    let body = br#"{"id":"CHAR_1","reference_id":"torg-5-key"}"#;
    let signature = "91e40d8ea36485a57fe87f33a193f206386b3e9948ddcd613d63819fe36b5107";

    assert_eq!(
        verify_pagseguro_signature("pagbank-test-token", signature, body),
        Ok(())
    );
    assert_eq!(
        verify_pagseguro_signature("wrong-token", signature, body),
        Err("Assinatura inválida")
    );
    assert_eq!(
        verify_pagseguro_signature(
            "pagbank-test-token",
            signature,
            br#"{"id":"CHAR_1", "reference_id":"torg-5-key"}"#
        ),
        Err("Assinatura inválida")
    );
}

#[test]
fn pagseguro_signature_header_requires_one_well_formed_digest() {
    const SIGNATURE: &str = "91e40d8ea36485a57fe87f33a193f206386b3e9948ddcd613d63819fe36b5107";
    let mut headers = axum::http::HeaderMap::new();

    assert_eq!(
        pagseguro_signature_header(&headers),
        Err("Assinatura inválida")
    );
    headers.insert(
        "x-authenticity-token",
        axum::http::HeaderValue::from_static(SIGNATURE),
    );
    assert_eq!(pagseguro_signature_header(&headers), Ok(SIGNATURE));
    headers.append(
        "x-authenticity-token",
        axum::http::HeaderValue::from_static(SIGNATURE),
    );
    assert_eq!(
        pagseguro_signature_header(&headers),
        Err("Assinatura inválida")
    );
    headers.remove("x-authenticity-token");
    headers.insert(
        "x-authenticity-token",
        axum::http::HeaderValue::from_static("not-a-digest"),
    );
    assert_eq!(
        pagseguro_signature_header(&headers),
        Err("Assinatura inválida")
    );
}

#[test]
fn reconciliation_window_defaults_to_three_hours_and_rejects_invalid_values() {
    assert_eq!(parse_window_hours(None), 3);
    assert_eq!(parse_window_hours(Some("")), 3);
    assert_eq!(parse_window_hours(Some("6")), 6);
    assert_eq!(parse_window_hours(Some("0")), 3);
    assert_eq!(parse_window_hours(Some("abc")), 3);
    assert_eq!(parse_window_hours(Some("9999")), 3);
}

#[test]
fn pagseguro_error_log_keeps_only_codes() {
    let body = r#"{"error_messages":[{"code":"40002","description":"card 4111111111111111","parameter_name":"holder"}]}"#;
    let codes = crate::infrastructure::pagseguro::PagSeguro::error_codes(body);
    assert_eq!(codes, "40002");
    assert!(!codes.contains("4111"));
    assert_eq!(
        crate::infrastructure::pagseguro::PagSeguro::error_codes("<html>card 4111</html>"),
        ""
    );
}

#[tokio::test]
async fn mock_cancel_voids_pending_and_refuses_captured() {
    use crate::infrastructure::mock_payment::{MockPaymentProvider, MockProviderKind};
    let mock = MockPaymentProvider::new(MockProviderKind::PagSeguro);
    let voided = mock
        .cancel("mock:pagseguro:pending:5000:BRL:torg-1-k")
        .await
        .unwrap();
    assert_eq!(voided.status, ProviderStatus::Failed);
    assert!(
        mock.cancel("mock:pagseguro:captured:5000:BRL:torg-1-k")
            .await
            .is_err()
    );
}

#[test]
fn mock_mode_requires_explicit_allow() {
    assert!(mock_mode_guard(None, None).is_ok());
    assert!(mock_mode_guard(Some("real"), None).is_ok());
    assert!(mock_mode_guard(Some("mock"), None).is_err());
    assert!(mock_mode_guard(Some("mock"), Some("1")).is_err());
    assert!(mock_mode_guard(Some("mock"), Some("true")).is_ok());
}
