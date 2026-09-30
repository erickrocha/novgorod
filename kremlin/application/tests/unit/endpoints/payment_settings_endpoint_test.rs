use super::*;

fn user(role: Role, tenant_id: Option<i64>) -> User {
    User {
        id: Some(1),
        uuid: None,
        name: None,
        email: "test@example.com".into(),
        password: String::new(),
        enabled: true,
        first_login: false,
        role,
        tenant_id,
        created_at: None,
        updated_at: None,
        created_by: None,
        updated_by: None,
    }
}

#[test]
fn only_matching_owner_and_sysadmin_can_manage_payment_settings() {
    assert!(authorize(&user(Role::SysAdmin, None), 10).is_ok());
    assert!(authorize(&user(Role::TenantOwner, Some(10)), 10).is_ok());
    assert!(authorize(&user(Role::TenantOwner, Some(11)), 10).is_err());
    assert!(authorize(&user(Role::TenantUser, Some(10)), 10).is_err());
    assert!(authorize(&user(Role::Customer, Some(10)), 10).is_err());
    assert!(authorize(&user(Role::TenantOwner, None), 10).is_err());
}

#[test]
fn payment_settings_response_exposes_status_and_metadata_only() {
    let json = serde_json::to_value(PaymentSettingsResponse {
        provider: "pagseguro".into(),
        version: 2,
        credentials_configured: true,
        public_key: Some("PS_PUBLIC_KEY".into()),
        environment: Some("sandbox".into()),
    })
    .unwrap();

    assert_eq!(json["provider"], "pagseguro");
    assert_eq!(json["version"], 2);
    assert_eq!(json["credentialsConfigured"], true);
    assert_eq!(json["publicKey"], "PS_PUBLIC_KEY");
    assert_eq!(json["environment"], "sandbox");
    assert!(json.get("credentials").is_none());
    assert!(json.get("token").is_none());
    assert!(json.get("accessToken").is_none());
}
