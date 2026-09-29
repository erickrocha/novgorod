use super::*;

#[test]
fn encryption_is_random_authenticated_and_tenant_bound() {
    let ring = ShippingKeyRing {
        active: "v1".into(),
        keys: BTreeMap::from([("v1".into(), [42; 32])]),
    };
    let credentials = CorreiosCredentials {
        username: "secret-user".into(),
        api_access_code: "secret-code".into(),
        ..Default::default()
    };
    let (version, mut bytes) = ring.encrypt(7, &credentials).unwrap();
    assert_ne!(bytes, ring.encrypt(7, &credentials).unwrap().1);
    assert_eq!(
        ring.decrypt(7, &version, &bytes).unwrap().username,
        "secret-user"
    );
    assert!(ring.decrypt(8, &version, &bytes).is_err());
    assert!(ring.decrypt(7, "v2", &bytes).is_err());
    bytes[15] ^= 1;
    assert!(ring.decrypt(7, &version, &bytes).is_err());
    assert!(!format!("{credentials:?}").contains("secret"));
}
