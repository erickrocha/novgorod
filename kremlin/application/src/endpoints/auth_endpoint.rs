use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::endpoints::json::access_token_json::AccessTokenJson;
use crate::endpoints::json::login_request::LoginRequest;
use crate::endpoints::json::refresh_token_request::RefreshTokenRequest;
use crate::infrastructure::mapper::{AccessTokenMapper, Mapper};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Form, Json};
use business::use_cases::account_invite_use_case::AccountInviteUseCase;
use business::use_cases::authentication_use_case::AuthenticationUseCase;
use business::use_cases::customer_registration_use_case::CustomerRegistrationUseCase;
use crate::endpoints::json::signup_request::SignupRequest;
use crate::endpoints::json::customer_json::CustomerJson;
use crate::infrastructure::mapper::CustomerMapper;
use business::domain::customer::Customer;
use business::domain::customer_address::CustomerAddress;
use business::domain::user::User;
use business::domain::enums::Role;

#[utoipa::path(
    post,
    path = "/login",
    request_body(content = LoginRequest, content_type = "application/x-www-form-urlencoded"),
    responses(
        (status = 200, description = "Login successful", body = AccessTokenJson),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn sign_in(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Form(login_request): Form<LoginRequest>,
) -> HttpResponse<Json<AccessTokenJson>> {
    let access_token =
        AuthenticationUseCase::execute(&state.conn, login_request.email, login_request.password)
            .await;
    match access_token {
        Ok(token) => Ok(Json(AccessTokenMapper::json(token))),
        Err(_) => Err(ExceptionResponse::Unauthorized(
            locale,
            ErrorKey::BadCredentials,
        )),
    }
}

#[utoipa::path(
    post,
    path = "/signup",
    request_body = SignupRequest,
    responses(
        (status = 200, description = "Registration successful", body = CustomerJson),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn signup(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Json(signup_request): Json<SignupRequest>,
) -> HttpResponse<Json<CustomerJson>> {
    // Basic mapping
    let user = User {
        id: None,
        uuid: None,
        tenant_id: None, // Customer registration is not tenant bound (for now)
        name: Some(signup_request.name.clone()),
        email: signup_request.email.clone(),
        password: signup_request.password.clone(),
        role: Role::Customer,
        enabled: true,
        first_login: true,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    };
    
    let customer = Customer {
        id: None,
        uuid: None,
        tenant_id: None,
        user_id: None,
        name: signup_request.name.clone(),
        email: signup_request.email.clone(),
        password_hash: signup_request.password.clone(),
        cpf: signup_request.cpf.clone(),
        phone: signup_request.phone.clone(),
        marketing_consent: false,
        consent_at: None,
        active: true,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    };
    
    let address = signup_request.address.map(|a| CustomerAddress {
        id: None,
        uuid: None,
        tenant_id: None,
        customer_id: 0, // Assigned inside usecase
        label: a.label,
        recipient: a.recipient,
        cep: a.cep,
        logradouro: a.logradouro,
        numero: a.numero,
        complemento: a.complemento,
        bairro: a.bairro,
        cidade: a.cidade,
        uf: a.uf,
        is_default: a.is_default.unwrap_or(true),
        created_at: None,
        updated_at: None,
        created_by: None,
        updated_by: None,
    });

    let result = CustomerRegistrationUseCase::execute(
        &state.conn,
        user,
        customer,
        address
    ).await;
    
    match result {
        Ok(c) => Ok(Json(CustomerMapper::json(c))),
        Err(_) => Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ))
    }
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AcceptInviteJson {
    pub token: String,
    pub new_password: String,
}

#[utoipa::path(
    post,
    path = "/accept-invite",
    tag = "Authentication",
    request_body = AcceptInviteJson,
    responses((status = 204, description = "Senha definida"))
)]
pub async fn accept_invite(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Json(payload): Json<AcceptInviteJson>,
) -> Result<StatusCode, ExceptionResponse> {
    match AccountInviteUseCase::accept(state.conn.as_ref(), &payload.token, &payload.new_password)
        .await
    {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        )),
    }
}

#[utoipa::path(
    post,
    path = "/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = AccessTokenJson),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn refresh_token(
    state: State<AppState>,
    Extension(locale): Extension<Locale>,
    Json(request): Json<RefreshTokenRequest>,
) -> HttpResponse<Json<AccessTokenJson>> {
    let access_token =
        AuthenticationUseCase::refresh_token(&state.conn, request.refresh_token).await;
    match access_token {
        Ok(token) => Ok(Json(AccessTokenMapper::json(token))),
        Err(_) => Err(ExceptionResponse::Unauthorized(
            locale,
            ErrorKey::BadCredentials,
        )),
    }
}
