//! Tenant context resolution (C-018, SR-TEN-001..004, 010..012, 015, 016).
//!
//! One middleware computes exactly one `TenantContext` per request and stores it as a request
//! extension. Handlers read the extension; no handler parses `x-tenant-id` itself.
use crate::AppState;
use crate::commons::exception_response::ExceptionResponse;
use crate::commons::i18n::{ErrorKey, Locale};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::header::ACCEPT_LANGUAGE;
use axum::{body::Body, extract::Request, http::Response, middleware::Next};
use business::gateway::tenant_gateway::TenantGateway;
use business::commons::gateway::Gateway;

pub const TENANT_HEADER: &str = "x-tenant-id";
pub const GATEWAY_TOKEN_HEADER: &str = "x-gateway-token";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TenantContext {
    /// No tenant on the request: marketplace mode, no tenant restriction.
    Marketplace,
    /// Client-supplied tenant, accepted only when the tenant is listed.
    Selector(i64),
    /// Tenant bound by the gateway (valid `x-gateway-token`).
    Fixed(i64),
}

impl TenantContext {
    pub fn tenant_id(&self) -> Option<i64> {
        match self {
            TenantContext::Marketplace => None,
            TenantContext::Selector(id) | TenantContext::Fixed(id) => Some(*id),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    /// More than one `x-tenant-id` header (SR-TEN-016): 400.
    DuplicateHeader,
    /// Selector not a number (SR-TEN-004): 400.
    MalformedSelector,
    /// Gateway proof present but invalid or empty (SR-TEN-011): 403.
    InvalidProof,
    /// Gateway request without a usable tenant (SR-TEN-012): 400.
    MissingGatewayTenant,
    /// Selector tenant unknown or not listed (SR-TEN-003): 400.
    UnknownOrUnlistedTenant,
    /// Gateway tenant does not exist (SR-TEN-012): 400.
    UnknownGatewayTenant,
}

impl Rejection {
    fn reason(&self) -> &'static str {
        match self {
            Rejection::DuplicateHeader => "duplicate_header",
            Rejection::MalformedSelector => "malformed_selector",
            Rejection::InvalidProof => "invalid_proof",
            Rejection::MissingGatewayTenant => "missing_gateway_tenant",
            Rejection::UnknownOrUnlistedTenant => "unknown_or_unlisted_tenant",
            Rejection::UnknownGatewayTenant => "unknown_gateway_tenant",
        }
    }

    fn response(&self, locale: Locale) -> ExceptionResponse {
        match self {
            Rejection::InvalidProof => {
                ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue)
            }
            _ => ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue),
        }
    }
}

/// What the headers alone decide; the database still has to confirm the tenant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Marketplace,
    /// Proof valid: tenant must exist, need not be listed (A10).
    Fixed(i64),
    /// No proof: tenant must exist and be listed.
    Selector(i64),
    Reject(Rejection),
}

/// Pure decision from the headers (DES-TEN-02). `secret` is the configured gateway token; when it
/// is not configured every proof is invalid (fail closed).
pub fn decide(headers: &HeaderMap, secret: Option<&str>) -> Decision {
    let mut values = headers.get_all(TENANT_HEADER).iter();
    let first = values.next();
    if values.next().is_some() {
        return Decision::Reject(Rejection::DuplicateHeader);
    }
    // an empty or whitespace-only value is treated as absent (A4)
    let selector = first
        .map(|v| v.to_str().map(str::trim))
        .transpose()
        .map(|v| v.filter(|s| !s.is_empty()));

    if let Some(proof) = headers.get(GATEWAY_TOKEN_HEADER) {
        let valid = match (secret, proof.to_str()) {
            (Some(expected), Ok(given)) => !given.is_empty() && constant_time_eq(expected, given),
            _ => false,
        };
        if !valid {
            return Decision::Reject(Rejection::InvalidProof);
        }
        return match selector {
            Ok(Some(s)) => match s.parse::<i64>() {
                Ok(id) => Decision::Fixed(id),
                Err(_) => Decision::Reject(Rejection::MissingGatewayTenant),
            },
            _ => Decision::Reject(Rejection::MissingGatewayTenant),
        };
    }

    match selector {
        Ok(None) => Decision::Marketplace,
        Ok(Some(s)) => match s.parse::<i64>() {
            Ok(id) => Decision::Selector(id),
            Err(_) => Decision::Reject(Rejection::MalformedSelector),
        },
        Err(_) => Decision::Reject(Rejection::MalformedSelector),
    }
}

/// Compares in time that depends only on the lengths, not on where the values differ.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = (a.len() ^ b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        diff |= a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0);
    }
    diff == 0
}

pub async fn tenant_context(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, ExceptionResponse> {
    let locale = Locale::from_accept_language(
        req.headers()
            .get(ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok()),
    );

    let decision = decide(req.headers(), state.gateway_token.as_deref());
    // the proof header never reaches the handlers (DES-TEN-05)
    req.headers_mut().remove(GATEWAY_TOKEN_HEADER);

    let outcome = match decision {
        Decision::Marketplace => Ok(TenantContext::Marketplace),
        Decision::Reject(r) => Err(r),
        Decision::Fixed(id) => match find_tenant(&state, id).await {
            Some(_) => Ok(TenantContext::Fixed(id)),
            None => Err(Rejection::UnknownGatewayTenant),
        },
        Decision::Selector(id) => match find_tenant(&state, id).await {
            Some(listed) if listed => Ok(TenantContext::Selector(id)),
            _ => Err(Rejection::UnknownOrUnlistedTenant),
        },
    };

    match outcome {
        Ok(context) => {
            req.extensions_mut().insert(context);
            Ok(next.run(req).await)
        }
        Err(rejection) => {
            // reason class and path only; never a credential value (SR-TEN-015)
            log::warn!(
                "tenant resolution rejected: reason={} path={}",
                rejection.reason(),
                req.uri().path()
            );
            Err(rejection.response(locale))
        }
    }
}

/// `Some(listed)` when the tenant exists, `None` when it does not (or the lookup fails: fail closed).
async fn find_tenant(state: &AppState, id: i64) -> Option<bool> {
    match TenantGateway::new(state.conn.as_ref().clone()).find_by_id(id).await {
        Ok(tenant) => tenant.map(|t| t.listed),
        Err(error) => {
            log::error!("tenant lookup failed during tenant resolution: {error}");
            None
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/commons/tenant_context_test.rs"]
mod tests;
