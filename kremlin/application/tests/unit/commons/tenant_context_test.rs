use super::*;
use axum::http::HeaderValue;

fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
    let mut h = HeaderMap::new();
    for (k, v) in pairs {
        h.append(*k, HeaderValue::from_str(v).unwrap());
    }
    h
}

#[test]
fn no_header_or_empty_header_is_marketplace() {
    assert_eq!(decide(&headers(&[]), None), Decision::Marketplace);
    assert_eq!(decide(&headers(&[(TENANT_HEADER, "")]), None), Decision::Marketplace);
    assert_eq!(decide(&headers(&[(TENANT_HEADER, "  ")]), None), Decision::Marketplace);
}

#[test]
fn selector_must_be_numeric() {
    assert_eq!(decide(&headers(&[(TENANT_HEADER, "7")]), None), Decision::Selector(7));
    assert_eq!(
        decide(&headers(&[(TENANT_HEADER, "abc")]), None),
        Decision::Reject(Rejection::MalformedSelector)
    );
}

#[test]
fn duplicate_header_is_rejected_even_when_identical_and_with_proof() {
    let dup = headers(&[(TENANT_HEADER, "7"), (TENANT_HEADER, "7")]);
    assert_eq!(decide(&dup, None), Decision::Reject(Rejection::DuplicateHeader));
    let dup_proof = headers(&[
        (TENANT_HEADER, "7"),
        (TENANT_HEADER, "8"),
        (GATEWAY_TOKEN_HEADER, "s3cret"),
    ]);
    assert_eq!(decide(&dup_proof, Some("s3cret")), Decision::Reject(Rejection::DuplicateHeader));
}

#[test]
fn valid_proof_binds_the_header_tenant() {
    let h = headers(&[(TENANT_HEADER, "7"), (GATEWAY_TOKEN_HEADER, "s3cret")]);
    assert_eq!(decide(&h, Some("s3cret")), Decision::Fixed(7));
}

#[test]
fn wrong_empty_or_unconfigured_proof_is_rejected_not_downgraded() {
    for proof in ["wrong", ""] {
        let h = headers(&[(TENANT_HEADER, "7"), (GATEWAY_TOKEN_HEADER, proof)]);
        assert_eq!(decide(&h, Some("s3cret")), Decision::Reject(Rejection::InvalidProof));
    }
    let h = headers(&[(TENANT_HEADER, "7"), (GATEWAY_TOKEN_HEADER, "s3cret")]);
    assert_eq!(decide(&h, None), Decision::Reject(Rejection::InvalidProof));
}

#[test]
fn proven_request_needs_a_numeric_tenant() {
    for pairs in [
        vec![(GATEWAY_TOKEN_HEADER, "s3cret")],
        vec![(GATEWAY_TOKEN_HEADER, "s3cret"), (TENANT_HEADER, "")],
        vec![(GATEWAY_TOKEN_HEADER, "s3cret"), (TENANT_HEADER, "abc")],
    ] {
        assert_eq!(
            decide(&headers(&pairs), Some("s3cret")),
            Decision::Reject(Rejection::MissingGatewayTenant)
        );
    }
}

#[test]
fn constant_time_eq_compares_content_and_length() {
    assert!(constant_time_eq("abc", "abc"));
    assert!(!constant_time_eq("abc", "abd"));
    assert!(!constant_time_eq("abc", "abcd"));
    assert!(!constant_time_eq("", "a"));
}
