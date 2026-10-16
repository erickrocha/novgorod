use crate::commons::i18n::{ErrorKey, Locale, translate};
use crate::endpoints::json::error_response_json::ErrorResponseJson;
use axum::Json;
use axum::response::IntoResponse;

pub type HttpResponse<T> = Result<T, ExceptionResponse>;

#[derive(Debug)]
pub enum ExceptionResponse {
    ServiceUnavailable,
    Unauthorized(Locale, ErrorKey),

    Forbidden(Locale, ErrorKey),

    BadRequest(Locale, ErrorKey),
    UnprocessableEntity(Locale, ErrorKey),

    NotFound(Locale, ErrorKey),

    CustomBadRequest(String),
    Conflict(Locale, ErrorKey),
    InternalServerError(Locale, ErrorKey),
    /// 429; the number is the `Retry-After` in seconds.
    TooManyRequests(Locale, ErrorKey, u64),
}

impl IntoResponse for ExceptionResponse {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        let (status, payload) = match self {
            Self::ServiceUnavailable => (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                ErrorResponseJson::new(
                    "shipping_unavailable".into(),
                    "Shipping temporarily unavailable; retry later".into(),
                ),
            ),
            ExceptionResponse::Unauthorized(locale, key) => (
                axum::http::StatusCode::UNAUTHORIZED,
                ErrorResponseJson::new(key.as_str().to_string(), translate(locale, key)),
            ),
            ExceptionResponse::Forbidden(locale, key) => (
                axum::http::StatusCode::FORBIDDEN,
                ErrorResponseJson::new(key.as_str().to_string(), translate(locale, key)),
            ),
            ExceptionResponse::BadRequest(locale, key) => (
                axum::http::StatusCode::BAD_REQUEST,
                ErrorResponseJson::new(key.as_str().to_string(), translate(locale, key)),
            ),
            ExceptionResponse::UnprocessableEntity(locale, key) => (
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                ErrorResponseJson::new(key.as_str().to_string(), translate(locale, key)),
            ),
            ExceptionResponse::NotFound(locale, key) => (
                axum::http::StatusCode::NOT_FOUND,
                ErrorResponseJson::new(key.as_str().to_string(), translate(locale, key)),
            ),
            ExceptionResponse::Conflict(locale, key) => (
                axum::http::StatusCode::CONFLICT,
                ErrorResponseJson::new(key.as_str().to_string(), translate(locale, key)),
            ),
            ExceptionResponse::InternalServerError(locale, key) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponseJson::new(key.as_str().to_string(), translate(locale, key)),
            ),
            ExceptionResponse::TooManyRequests(locale, key, retry_after) => {
                let body = ErrorResponseJson::new(key.as_str().to_string(), translate(locale, key));
                let mut response = (axum::http::StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();
                if let Ok(value) = axum::http::HeaderValue::from_str(&retry_after.to_string()) {
                    response.headers_mut().insert(axum::http::header::RETRY_AFTER, value);
                }
                return response;
            }
            ExceptionResponse::CustomBadRequest(msg) => (
                axum::http::StatusCode::BAD_REQUEST,
                ErrorResponseJson::new("bad_request".to_string(), msg),
            ),
        };

        (status, Json(payload)).into_response()
    }
}
