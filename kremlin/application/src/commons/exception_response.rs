use crate::commons::i18n::{ErrorKey, Locale, translate};
use crate::endpoints::json::error_response_json::ErrorResponseJson;
use axum::Json;
use axum::response::IntoResponse;

pub type HttpResponse<T> = Result<T, ExceptionResponse>;

#[derive(Debug)]
pub enum ExceptionResponse {
    Unauthorized(Locale, ErrorKey),

    Forbidden(Locale, ErrorKey),

    BadRequest(Locale, ErrorKey),

    NotFound(Locale, ErrorKey),

    CustomBadRequest(String),
}

impl IntoResponse for ExceptionResponse {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        let (status, payload) = match self {
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
            ExceptionResponse::NotFound(locale, key) => (
                axum::http::StatusCode::NOT_FOUND,
                ErrorResponseJson::new(key.as_str().to_string(), translate(locale, key)),
            ),
            ExceptionResponse::CustomBadRequest(msg) => (
                axum::http::StatusCode::BAD_REQUEST,
                ErrorResponseJson::new("bad_request".to_string(), msg),
            ),
        };

        (status, Json(payload)).into_response()
    }
}
