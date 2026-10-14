use super::*;

#[test]
fn payment_credentials_encryption_is_authenticated_and_tenant_bound() {
    let ring = PaymentKeyRing {
        active: "v1".into(),
        keys: BTreeMap::from([("v1".into(), [42; 32])]),
    };

    let creds = TenantCredentialsPayload::PagSeguro(PagSeguroCredentials {
        token: "PAGSEGURO_SECRET_TOKEN".into(),
        public_key: Some("PAGSEGURO_PUBLIC_KEY".into()),
        environment: Some("sandbox".into()),
    });

    let (version, mut bytes) = ring.encrypt_payload(10, &creds).unwrap();
    assert_ne!(bytes, ring.encrypt_payload(10, &creds).unwrap().1);

    match ring.decrypt_payload(10, &version, &bytes).unwrap() {
        TenantCredentialsPayload::PagSeguro(c) => {
            assert_eq!(c.token, "PAGSEGURO_SECRET_TOKEN");
            assert_eq!(c.public_key.as_deref(), Some("PAGSEGURO_PUBLIC_KEY"));
            assert_eq!(c.environment.as_deref(), Some("sandbox"));
        }
    }

    // Tenant isolation
    assert!(ring.decrypt_payload(11, &version, &bytes).is_err());
    // Key version mismatch
    assert!(ring.decrypt_payload(10, "v2", &bytes).is_err());
    // Tampered ciphertext
    bytes[15] ^= 1;
    assert!(ring.decrypt_payload(10, &version, &bytes).is_err());

    assert!(!format!("{:?}", PagSeguroCredentials { token: "secret".into(), ..Default::default() }).contains("secret"));
}
