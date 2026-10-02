use super::*;

#[test]
fn resolved_tenant_is_anded_with_the_query_tenant() {
    assert_eq!(scope_tenant(TenantContext::Marketplace, None), Ok(None));
    assert_eq!(scope_tenant(TenantContext::Marketplace, Some(2)), Ok(Some(2)));
    assert_eq!(scope_tenant(TenantContext::Selector(1), None), Ok(Some(1)));
    assert_eq!(scope_tenant(TenantContext::Selector(1), Some(1)), Ok(Some(1)));
    assert_eq!(scope_tenant(TenantContext::Fixed(1), Some(2)), Err(()));
}
