use crate::AppState;
use crate::commons::{
    exception_response::{ExceptionResponse, HttpResponse},
    i18n::{ErrorKey, Locale},
};
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use business::{
    domain::{enums::Role, marketplace::PurchaseError, user::User},
    use_cases::checkout_quote_use_case::{CheckoutQuoteUseCase, QuoteRequest, QuoteResult},
};

pub(crate) fn error(locale: Locale, error: PurchaseError) -> ExceptionResponse {
    match error {
        PurchaseError::DeliveryRateMissing(msg) => ExceptionResponse::CustomBadRequest(msg),
        PurchaseError::Validation(msg) => ExceptionResponse::CustomBadRequest(msg.into()),
        PurchaseError::ShippingUnavailable => ExceptionResponse::ServiceUnavailable,
        PurchaseError::NotFound => ExceptionResponse::NotFound(locale, ErrorKey::PurchaseNotFound),
        PurchaseError::Forbidden => {
            ExceptionResponse::Forbidden(locale, ErrorKey::PurchaseForbidden)
        }
        PurchaseError::Conflict => ExceptionResponse::Conflict(locale, ErrorKey::PurchaseConflict),
        PurchaseError::Persistence(e) => {
            log::error!("Checkout quote failed: {e}");
            ExceptionResponse::InternalServerError(locale, ErrorKey::PurchaseUnavailable)
        }
    }
}

#[utoipa::path(post, path="/checkout/quotes", request_body=QuoteRequest, responses((status=201, body=QuoteResult), (status=400, description="Invalid address, parcel, or unavailable delivery"), (status=503, description="Shipping provider temporarily unavailable; retry")), security(("bearer_auth"=[])), tag="Shipping")]
pub async fn quote(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Extension(context): Extension<crate::commons::tenant_context::TenantContext>,
    Json(input): Json<QuoteRequest>,
) -> HttpResponse<(StatusCode, Json<QuoteResult>)> {
    if user.role != Role::Customer {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::PurchaseForbidden,
        ));
    }
    let user_id = user.id.ok_or(ExceptionResponse::Forbidden(
        locale,
        ErrorKey::PurchaseForbidden,
    ))?;
    let result =
        CheckoutQuoteUseCase::with_provider(state.conn.as_ref().clone(), state.shipping.clone())
            .create_scoped(user_id, input, context.tenant_id())
            .await
            .map_err(|e| error(locale, e))?;
    Ok((StatusCode::CREATED, Json(result)))
}

#[utoipa::path(post, path="/checkout/quotes/{id}/shipping-selection", params(("id"=i64, Path)), request_body=Vec<business::use_cases::checkout_quote_use_case::ShippingSelection>, responses((status=201, body=QuoteResult), (status=400, description="Invalid option selection"), (status=404, description="Quote not found or belongs to another customer"), (status=409, description="Expired quote or changed inputs")), security(("bearer_auth"=[])), tag="Shipping")]
pub async fn select_shipping(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Extension(context): Extension<crate::commons::tenant_context::TenantContext>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(input): Json<Vec<business::use_cases::checkout_quote_use_case::ShippingSelection>>,
) -> HttpResponse<(StatusCode, Json<QuoteResult>)> {
    if user.role != Role::Customer {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::PurchaseForbidden,
        ));
    }
    let user_id = user.id.ok_or(ExceptionResponse::Forbidden(
        locale,
        ErrorKey::PurchaseForbidden,
    ))?;
    ensure_quote_tenant(&state, id, context).await?;
    let result = CheckoutQuoteUseCase::new(state.conn.as_ref().clone())
        .select(user_id, id, input)
        .await
        .map_err(|e| error(locale, e))?;
    Ok((StatusCode::CREATED, Json(result)))
}

/// SR-TEN-007: with a Selector/Fixed context a quote holding items of another tenant is refused.
pub(crate) async fn ensure_quote_tenant(
    state: &AppState,
    quote_id: i64,
    context: crate::commons::tenant_context::TenantContext,
) -> Result<(), ExceptionResponse> {
    let Some(tenant_id) = context.tenant_id() else {
        return Ok(());
    };
    match CheckoutQuoteUseCase::belongs_to_tenant(state.conn.as_ref(), quote_id, tenant_id).await {
        Some(false) => Err(ExceptionResponse::CustomBadRequest(
            "item of another tenant".into(),
        )),
        _ => Ok(()),
    }
}
