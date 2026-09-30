use super::*;

#[test]
fn payment_credentials_encryption_is_authenticated_and_tenant_bound() {
    let ring = PaymentKeyRing {
        active: "v1".into(),
        keys: BTreeMap::from([("v1".into(), [42; 32])]),
    };

    let mp_creds = TenantCredentialsPayload::MercadoPago(MercadoPagoCredentials {
        access_token: "TEST-TOKEN-1234".into(),
        public_key: Some("TEST-PUBLIC-KEY".into()),
        collector_id: Some(12345),
        webhook_secret: Some("secret123".into()),
    });

    let (version, mut bytes) = ring.encrypt_payload(10, &mp_creds).unwrap();
    assert_ne!(bytes, ring.encrypt_payload(10, &mp_creds).unwrap().1);

    let decrypted = ring.decrypt_payload(10, &version, &bytes).unwrap();
    match decrypted {
        TenantCredentialsPayload::MercadoPago(c) => {
            assert_eq!(c.access_token, "TEST-TOKEN-1234");
            assert_eq!(c.public_key.as_deref(), Some("TEST-PUBLIC-KEY"));
            assert_eq!(c.collector_id, Some(12345));
        }
        _ => panic!("Expected MercadoPago payload"),
    }

    // Tenant isolation
    assert!(ring.decrypt_payload(11, &version, &bytes).is_err());
    // Key version mismatch
    assert!(ring.decrypt_payload(10, "v2", &bytes).is_err());
    // Tampered ciphertext
    bytes[15] ^= 1;
    assert!(ring.decrypt_payload(10, &version, &bytes).is_err());

    let ps_creds = TenantCredentialsPayload::PagSeguro(PagSeguroCredentials {
        token: "PAGSEGURO_SECRET_TOKEN".into(),
        public_key: Some("PAGSEGURO_PUBLIC_KEY".into()),
        environment: Some("sandbox".into()),
    });

    let (ps_version, ps_bytes) = ring.encrypt_payload(15, &ps_creds).unwrap();
    let ps_decrypted = ring.decrypt_payload(15, &ps_version, &ps_bytes).unwrap();
    match ps_decrypted {
        TenantCredentialsPayload::PagSeguro(c) => {
            assert_eq!(c.token, "PAGSEGURO_SECRET_TOKEN");
            assert_eq!(c.public_key.as_deref(), Some("PAGSEGURO_PUBLIC_KEY"));
            assert_eq!(c.environment.as_deref(), Some("sandbox"));
        }
        _ => panic!("Expected PagSeguro payload"),
    }

    assert!(!format!("{:?}", MercadoPagoCredentials { access_token: "secret".into(), ..Default::default() }).contains("secret"));
    assert!(!format!("{:?}", PagSeguroCredentials { token: "secret".into(), ..Default::default() }).contains("secret"));
}
