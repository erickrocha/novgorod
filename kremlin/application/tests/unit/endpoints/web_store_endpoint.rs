use super::{WebStorePageQuery, scoped};
use crate::commons::tenant_context::TenantContext;

#[test]
fn accepts_current_and_legacy_filter_parameter_names() {
    for query in [
        r#"{"minPrice":100,"maxPrice":500,"sortBy":"price-asc"}"#,
        r#"{"min_price":100,"max_price":500,"sort_by":"price-asc"}"#,
    ] {
        let parsed: WebStorePageQuery = serde_json::from_str(query).unwrap();
        assert_eq!(parsed.min_price, Some(100));
        assert_eq!(parsed.max_price, Some(500));
        assert_eq!(parsed.sort_by.as_deref(), Some("price-asc"));
    }
}

#[tokio::test]
async fn scoped_sets_the_row_level_scope_of_the_resolved_tenant_only() {
    use entity::audit_entity::{TenantScope, tenant_scope};
    assert_eq!(
        scoped(TenantContext::Selector(7), async { tenant_scope() }).await,
        TenantScope::Tenant(7)
    );
    assert_eq!(
        scoped(TenantContext::Fixed(9), async { tenant_scope() }).await,
        TenantScope::Tenant(9)
    );
    assert_eq!(
        scoped(TenantContext::Marketplace, async { tenant_scope() }).await,
        TenantScope::Unrestricted
    );
}
