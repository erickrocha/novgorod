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
fn only_matching_owner_and_sysadmin_can_manage_settings() {
    assert!(authorize(&user(Role::SysAdmin, None), 7).is_ok());
    assert!(authorize(&user(Role::TenantOwner, Some(7)), 7).is_ok());
    assert!(authorize(&user(Role::TenantOwner, Some(8)), 7).is_err());
    assert!(authorize(&user(Role::TenantUser, Some(7)), 7).is_err());
    assert!(authorize(&user(Role::Customer, Some(7)), 7).is_err());
    assert!(authorize(&user(Role::TenantOwner, None), 7).is_err());
}

#[test]
fn settings_response_exposes_status_only() {
    let json = serde_json::to_value(ShippingSettingsResponse {
        configuration: ShippingConfig::default(),
        version: 1,
        credentials_configured: true,
    })
    .unwrap();
    assert_eq!(json["credentialsConfigured"], true);
    assert!(json.get("credentials").is_none());
    assert!(json.get("keyVersion").is_none());
}
