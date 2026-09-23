use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::customer_json::*;
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::infrastructure::mapper::{CustomerMapper, Mapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::customer::CustomerEntityMapper;
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::customer_gateway::CustomerGateway;
use business::sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::customer_use_case::CustomerUseCase;
use entity::customer_entity;

const CUSTOMER_SORT_FIELDS: &[&str] = &[
    "id",
    "name",
    "email",
    "active",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/customers",
    tag = "Customer",
    responses(
        (status = 200, description = "List of customers", body = [CustomerJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<CustomerJson>> {
    let use_case = CustomerUseCase::new(CustomerGateway::new(state.conn.as_ref().clone()));
    let items = use_case.find_all().await;
    Json(CustomerMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/customers/paged",
    tag = "Customer",
    params(CustomerPageQuery),
    responses(
        (status = 200, description = "Paged customers", body = PagedResponse<CustomerJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CustomerPageQuery>,
) -> Json<PagedResponse<CustomerJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), CUSTOMER_SORT_FIELDS, "name");
    let mut query = customer_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(customer_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(customer_entity::Column::TenantId.eq(tid));
    }

    if let Some(active) = params.active {
        query = query.filter(customer_entity::Column::Active.eq(active));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(customer_entity::Column::Name.like(&pattern))
                .add(customer_entity::Column::Email.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "email" => customer_entity::Column::Email,
        "active" => customer_entity::Column::Active,
        "createdat" | "created_at" => customer_entity::Column::CreatedAt,
        "id" => customer_entity::Column::Id,
        _ => customer_entity::Column::Name,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(customer_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(customer_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = CustomerEntityMapper::from_models(rows);
    let items = CustomerMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/customers/{id}",
    tag = "Customer",
    params(("id" = i64, Path, description = "Customer ID")),
    responses(
        (status = 200, description = "Customer found", body = CustomerJson),
        (status = 404, description = "Not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_by_id(State(state): State<AppState>,Extension(locale): Extension<Locale>,Path(id): Path<i64>) -> HttpResponse<Json<CustomerJson>> {
    let use_case = CustomerUseCase::new(CustomerGateway::new(state.conn.as_ref().clone()));
    let item = use_case.find_by_id(id).await;
    match item {
        Some(item) => Ok(Json(CustomerMapper::json(item))),
        None => Err(ExceptionResponse::NotFound(locale,ErrorKey::InvalidParameterValue))
    }
}

#[utoipa::path(
    post,
    path = "/customers",
    tag = "Customer",
    request_body = CustomerJson,
    responses(
        (status = 201, description = "Customer created", body = CustomerJson),
        (status = 400, description = "Bad request", body = BadRequestErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn add(State(state): State<AppState>,Extension(locale): Extension<Locale>,Json(payload): Json<CustomerJson>) -> HttpResponse<(StatusCode, Json<CustomerJson>)> {
    if payload.name.trim().is_empty() || payload.email.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(locale,ErrorKey::InvalidParameterValue));
    }

    let customer = CustomerMapper::domain(payload);
    let use_case = CustomerUseCase::new(CustomerGateway::new(state.conn.as_ref().clone()));
    let saved = use_case.create(customer).await;

    match saved {
        Some(domain) => Ok((StatusCode::CREATED, Json(CustomerMapper::json(domain)))),
        None => Err(ExceptionResponse::BadRequest(locale,ErrorKey::InvalidParameterValue))
    }
}

#[utoipa::path(
    put,
    path = "/customers/{id}",
    tag = "Customer",
    params(("id" = i64, Path, description = "Customer ID")),
    request_body = CustomerJson,
    responses(
        (status = 200, description = "Customer updated", body = CustomerJson),
        (status = 400, description = "Bad request", body = BadRequestErrorJson),
        (status = 404, description = "Not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(State(state): State<AppState>,Extension(locale): Extension<Locale>,Path(id): Path<i64>,Json(payload): Json<CustomerJson>) -> HttpResponse<Json<CustomerJson>> {
    let use_case = CustomerUseCase::new(CustomerGateway::new(state.conn.as_ref().clone()));
    let customer = CustomerMapper::domain(payload);

    let saved = use_case.update(id, customer).await;

    match saved {
        Some(saved) => Ok(Json(CustomerMapper::json(saved))),
        None => Err(ExceptionResponse::BadRequest(locale,ErrorKey::InvalidParameterValue))
    }
}
