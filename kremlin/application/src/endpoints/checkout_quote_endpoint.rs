use crate::AppState;
use crate::commons::{exception_response::{ExceptionResponse, HttpResponse}, i18n::{ErrorKey, Locale}};
use axum::{Json, extract::{Extension, State}, http::StatusCode};
use business::{domain::{enums::Role, marketplace::PurchaseError, user::User}, use_cases::checkout_quote_use_case::{CheckoutQuoteUseCase, QuoteRequest, QuoteResult}};

fn error(locale: Locale, error: PurchaseError) -> ExceptionResponse {
    match error {
        PurchaseError::DeliveryRateMissing(msg) => ExceptionResponse::CustomBadRequest(msg),
        PurchaseError::Validation(_) => ExceptionResponse::BadRequest(locale, ErrorKey::PurchaseInvalid),
        PurchaseError::NotFound => ExceptionResponse::NotFound(locale, ErrorKey::PurchaseNotFound),
        PurchaseError::Forbidden => ExceptionResponse::Forbidden(locale, ErrorKey::PurchaseForbidden),
        PurchaseError::Conflict => ExceptionResponse::Conflict(locale, ErrorKey::PurchaseConflict),
        PurchaseError::Persistence(e) => { log::error!("Checkout quote failed: {e}"); ExceptionResponse::InternalServerError(locale, ErrorKey::PurchaseUnavailable) }
    }
}

pub async fn quote(State(state): State<AppState>, Extension(user): Extension<User>, Extension(locale): Extension<Locale>,
    Json(input): Json<QuoteRequest>) -> HttpResponse<(StatusCode, Json<QuoteResult>)> {
    if user.role != Role::Customer { return Err(ExceptionResponse::Forbidden(locale, ErrorKey::PurchaseForbidden)); }
    let user_id = user.id.ok_or(ExceptionResponse::Forbidden(locale, ErrorKey::PurchaseForbidden))?;
    let result = CheckoutQuoteUseCase::new(state.conn.as_ref().clone()).create(user_id, input).await.map_err(|e| error(locale, e))?;
    Ok((StatusCode::CREATED, Json(result)))
}
