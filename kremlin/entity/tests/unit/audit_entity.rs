    use super::{AuditUser, TenantScope, run_with_user, tenant_scope};

    #[tokio::test]
    async fn derives_tenant_scope_from_authenticated_user() {
        let user = AuditUser {
            id: 1,
            email: "owner@example.com".to_string(),
            tenant_id: Some(42),
            enforce_tenant: true,
        };
        let scope = run_with_user(Some(user), async { tenant_scope() }).await;
        assert_eq!(scope, TenantScope::Tenant(42));
    }

    #[tokio::test]
    async fn sysadmin_scope_is_unrestricted() {
        let user = AuditUser {
            id: 1,
            email: "admin@example.com".to_string(),
            tenant_id: None,
            enforce_tenant: false,
        };
        let scope = run_with_user(Some(user), async { tenant_scope() }).await;
        assert_eq!(scope, TenantScope::Unrestricted);
    }
