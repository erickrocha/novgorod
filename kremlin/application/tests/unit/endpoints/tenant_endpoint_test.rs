use super::*;

fn user(role: Role, tenant_id: Option<i64>) -> User {
    User {
        id: None,
        uuid: None,
        email: String::new(),
        name: None,
        password: String::new(),
        enabled: true,
        first_login: false,
        tenant_id,
        role,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    }
}

#[test]
fn sysadmin_sets_any_tenant_and_owner_only_its_own() {
    assert_eq!(listing_access(&user(Role::SysAdmin, None), 7), ListingAccess::Allowed);
    assert_eq!(listing_access(&user(Role::TenantOwner, Some(7)), 7), ListingAccess::Allowed);
    assert_eq!(listing_access(&user(Role::TenantOwner, Some(8)), 7), ListingAccess::NotFound);
}

#[test]
fn tenant_user_is_refused_and_customer_sees_not_found() {
    assert_eq!(listing_access(&user(Role::TenantUser, Some(7)), 7), ListingAccess::Forbidden);
    assert_eq!(listing_access(&user(Role::TenantUser, Some(8)), 7), ListingAccess::NotFound);
    assert_eq!(listing_access(&user(Role::Customer, None), 7), ListingAccess::NotFound);
}

#[tokio::test]
async fn listing_read_distinguishes_disabled_enabled_and_missing() {
    use business::sea_orm::{DbBackend, MockDatabase, Value};
    use std::collections::BTreeMap;

    for listed in [false, true] {
        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[BTreeMap::from([(
                "listed".to_string(),
                Value::Bool(Some(listed)),
            )])]])
            .into_connection();
        assert_eq!(TenantGateway::new(db).get_listed(7).await.unwrap(), Some(listed));
    }
    let db = MockDatabase::new(DbBackend::Postgres)
        .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
        .into_connection();
    assert_eq!(TenantGateway::new(db).get_listed(7).await.unwrap(), None);
}

#[tokio::test]
async fn listing_read_does_not_turn_database_errors_into_disabled_flags() {
    use business::sea_orm::{DbBackend, DbErr, MockDatabase};

    let db = MockDatabase::new(DbBackend::Postgres)
        .append_query_errors([DbErr::Custom("database unavailable".into())])
        .into_connection();
    assert!(TenantGateway::new(db).get_listed(7).await.is_err());
}
