use super::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;

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
