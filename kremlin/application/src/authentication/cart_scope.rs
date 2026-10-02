//! Cart-route scope for a Customer (C-018, SR-TEN-006, SR-TEN-021, DES-TEN-07).
//!
//! Runs after `authentication` on the cart routes only. For a Customer it finds the customer row
//! by the user id (one to one, never narrowed by a tenant filter) and re-scopes the row-level
//! hook to the resolved tenant: store mode enforces that tenant on cart rows, marketplace leaves
//! the tenant null and unrestricted. Other roles and all other routes keep their behavior.
use crate::AppState;
use crate::commons::exception_response::ExceptionResponse;
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::tenant_context::TenantContext;
use axum::extract::State;
use axum::{body::Body, extract::Request, http::Response, middleware::Next};
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::purchase_gateway::PurchaseGateway;

/// The customer that owns the carts of this request; `None` for staff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CartOwner(pub Option<i64>);

pub async fn cart_scope(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, ExceptionResponse> {
    let locale = req
        .extensions()
        .get::<Locale>()
        .copied()
        .unwrap_or(Locale::from_accept_language(None));
    let user = req.extensions().get::<User>().cloned();
    let context = req
        .extensions()
        .get::<TenantContext>()
        .copied()
        .unwrap_or(TenantContext::Marketplace);

    let Some(user) = user.filter(|u| u.role == Role::Customer) else {
        req.extensions_mut().insert(CartOwner(None));
        return Ok(next.run(req).await);
    };

    // zero or more than one customer for the user: refused (403), as `customer_for_user` does
    let customer = PurchaseGateway::customer_for_user(state.conn.as_ref(), user.id.unwrap_or(0))
        .await
        .map_err(|_| ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue))?;
    req.extensions_mut().insert(CartOwner(Some(customer.id)));

    let audit_user = entity::audit_entity::AuditUser {
        id: user.id.unwrap_or(0),
        email: user.email.clone(),
        tenant_id: context.tenant_id(),
        enforce_tenant: context.tenant_id().is_some(),
    };
    Ok(entity::audit_entity::run_with_user(Some(audit_user), next.run(req)).await)
}
