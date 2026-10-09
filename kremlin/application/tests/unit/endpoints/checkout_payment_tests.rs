use super::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;

#[test]
fn mock_provider_mode_is_opt_in_and_validates_provider() {
    assert!(mock_provider_for_config(None, None).unwrap().is_none());
    assert!(mock_provider_for_config(Some("real"), Some("invalid")).unwrap().is_none());
    assert!(mock_provider_for_config(Some("unknown"), None).is_err());
    assert!(mock_provider_for_config(Some("mock"), None).is_err());
    assert!(mock_provider_for_config(Some("mock"), Some("invalid")).is_err());

    assert_eq!(
        mock_provider_for_config(Some("mock"), Some("mercado_pago"))
            .unwrap()
            .unwrap()
            .name(),
        "mercado_pago"
    );
    assert_eq!(
        mock_provider_for_config(Some("mock"), Some("pagseguro"))
            .unwrap()
            .unwrap()
            .name(),
        "pagseguro"
    );
}

fn compute_signature(secret: &str, ts: &str, request_id: &str, data_id: &str) -> String {
    let manifest = format!("id:{data_id};request-id:{request_id};ts:{ts};");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("valid HMAC key");
    mac.update(manifest.as_bytes());
    let hash = mac.finalize().into_bytes();
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    format!("ts={ts},v1={hex}")
}

#[test]
fn test_verify_signature_success() {
    let secret = "super-secret-key-123";
    let data_id = "99887766";
    let request_id = "req-uuid-456";
    let ts = "1727280000";

    let sig = compute_signature(secret, ts, request_id, data_id);
    let result = verify_signature(secret, &sig, request_id, data_id);
    assert!(result.is_ok());
}

#[test]
fn test_verify_signature_wrong_secret_fails() {
    let secret = "correct-secret";
    let wrong_secret = "wrong-secret";
    let data_id = "99887766";
    let request_id = "req-uuid-456";
    let ts = "1727280000";

    let sig = compute_signature(secret, ts, request_id, data_id);
    let result = verify_signature(wrong_secret, &sig, request_id, data_id);
    assert_eq!(result, Err("Assinatura inválida"));
}

#[test]
fn test_verify_signature_tampered_data_id_fails() {
    let secret = "secret-123";
    let data_id = "99887766";
    let tampered_data_id = "99887767";
    let request_id = "req-uuid-456";
    let ts = "1727280000";

    let sig = compute_signature(secret, ts, request_id, data_id);
    let result = verify_signature(secret, &sig, request_id, tampered_data_id);
    assert_eq!(result, Err("Assinatura inválida"));
}

#[test]
fn test_verify_signature_tampered_timestamp_fails() {
    let secret = "secret-123";
    let data_id = "99887766";
    let request_id = "req-uuid-456";
    let ts = "1727280000";

    let sig = compute_signature(secret, ts, request_id, data_id);
    // Replace ts with different value in header
    let tampered_sig = sig.replace("1727280000", "1727280001");
    let result = verify_signature(secret, &tampered_sig, request_id, data_id);
    assert_eq!(result, Err("Assinatura inválida"));
}

#[test]
fn test_verify_signature_malformed_header_fails() {
    let secret = "secret-123";
    assert_eq!(
        verify_signature(secret, "malformed", "req-1", "123"),
        Err("Assinatura inválida")
    );
    assert_eq!(
        verify_signature(secret, "ts=12345", "req-1", "123"),
        Err("Assinatura inválida")
    );
    assert_eq!(
        verify_signature(secret, "v1=abcdef", "req-1", "123"),
        Err("Assinatura inválida")
    );
}

fn mp_payload(token: &str) -> TenantCredentialsPayload {
    TenantCredentialsPayload::MercadoPago(crate::infrastructure::payment_credentials::MercadoPagoCredentials {
        access_token: token.into(),
        public_key: None,
        collector_id: Some(7),
        webhook_secret: Some("whsec".into()),
    })
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
    let mp = build_tenant_provider("mercado_pago", mp_payload("tok")).unwrap();
    assert_eq!(mp.name(), "mercado_pago");
    assert_eq!(mp.collector_id(), 7);
    let ps = build_tenant_provider("pagseguro", ps_payload("tok", Some("sandbox"))).unwrap();
    assert_eq!(ps.name(), "pagseguro");
    assert!(build_tenant_provider("pagseguro", ps_payload("tok", Some("production"))).is_ok());
}

#[test]
fn tenant_provider_rejects_column_mismatch_and_invalid_credentials() {
    assert!(build_tenant_provider("pagseguro", mp_payload("tok")).is_err());
    assert!(build_tenant_provider("mercado_pago", ps_payload("tok", Some("sandbox"))).is_err());
    assert!(build_tenant_provider("mercado_pago", mp_payload("  ")).is_err());
    assert!(build_tenant_provider("pagseguro", ps_payload("", Some("sandbox"))).is_err());
}

#[test]
fn pagseguro_environment_is_validated_not_defaulted_to_sandbox() {
    assert!(build_tenant_provider("pagseguro", ps_payload("tok", None)).is_err());
    assert!(build_tenant_provider("pagseguro", ps_payload("tok", Some("prod"))).is_err());
    assert!(build_tenant_provider("pagseguro", ps_payload("tok", Some(""))).is_err());
}

// NOV-5 TC-22 (SR-PAY-020): numeric ids are read without leaking per call.
#[test]
fn notification_data_id_reads_query_and_numeric_or_string_body_ids() {
    assert_eq!(notification_data_id(Some("11".into()), b""), Some("11".into()));
    assert_eq!(notification_data_id(None, br#"{"data":{"id":123}}"#), Some("123".into()));
    assert_eq!(notification_data_id(None, br#"{"data":{"id":"456"}}"#), Some("456".into()));
    assert_eq!(notification_data_id(None, br#"{"id":789}"#), Some("789".into()));
    assert_eq!(notification_data_id(None, b"not json"), None);
    assert_eq!(notification_data_id(None, b""), None);
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
    let mock = MockPaymentProvider::new(MockProviderKind::MercadoPago);
    let voided = mock
        .cancel("mock:mercado_pago:pending:5000:BRL:torg-1-k")
        .await
        .unwrap();
    assert_eq!(voided.status, ProviderStatus::Failed);
    assert!(
        mock.cancel("mock:mercado_pago:captured:5000:BRL:torg-1-k")
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
