use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::customer_json::*;
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::infrastructure::mapper::{CustomerAddressMapper, Mapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::customer_address::CustomerAddressEntityMapper;
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::customer_address_gateway::CustomerAddressGateway;
use business::sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};
use business::use_cases::customer_address_use_case::CustomerAddressUseCase;
use entity::customer_address_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser | Role::Customer => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const CUSTOMER_ADDRESS_SORT_FIELDS: &[&str] =
    &["id", "recipient", "cidade", "uf", "createdAt", "created_at"];

#[utoipa::path(
    get,
    path = "/customer-addresses",
    tag = "Customer",
    responses(
        (status = 200, description = "List of customer addresses", body = [CustomerAddressJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<CustomerAddressJson>> {
    let use_case =
        CustomerAddressUseCase::new(CustomerAddressGateway::new(state.conn.as_ref().clone()));
    let items = use_case.find_all().await;
    Json(CustomerAddressMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/customer-addresses/paged",
    tag = "Customer",
    params(CustomerAddressPageQuery),
    responses(
        (status = 200, description = "Paged customer addresses", body = PagedResponse<CustomerAddressJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CustomerAddressPageQuery>,
) -> Json<PagedResponse<CustomerAddressJson>> {
    let norm =
        NormalizedPagination::new(&params.to_page_query(), CUSTOMER_ADDRESS_SORT_FIELDS, "id");
    let mut query = customer_address_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(customer_address_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(customer_address_entity::Column::TenantId.eq(tid));
    }

    if let Some(cid) = params.customer_id {
        query = query.filter(customer_address_entity::Column::CustomerId.eq(cid));
    }

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = CustomerAddressEntityMapper::from_models(rows);
    let items = CustomerAddressMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/customer-addresses/{id}",
    tag = "Customer",
    params(("id" = i64, Path, description = "Customer Address ID")),
    responses(
        (status = 200, description = "Customer address found", body = CustomerAddressJson),
        (status = 404, description = "Not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_by_id(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(current_user): Extension<User>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<CustomerAddressJson>> {
    let usecase =
        CustomerAddressUseCase::new(CustomerAddressGateway::new(state.conn.as_ref().clone()));
    let item = usecase
        .find_by_id(id)
        .await
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if !can_read_tenant(&current_user, item.tenant_id) {
        return Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    Ok(Json(CustomerAddressMapper::json(item)))
}

#[utoipa::path(
    post,
    path = "/customer-addresses",
    tag = "Customer",
    request_body = CustomerAddressJson,
    responses(
        (status = 201, description = "Customer address created", body = CustomerAddressJson),
        (status = 400, description = "Bad request", body = BadRequestErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn add(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Json(payload): Json<CustomerAddressJson>,
) -> HttpResponse<(StatusCode, Json<CustomerAddressJson>)> {
    let domain = CustomerAddressMapper::domain(payload);

    let use_case =
        CustomerAddressUseCase::new(CustomerAddressGateway::new(state.conn.as_ref().clone()));
    let saved = use_case
        .persist(domain)
        .await
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    Ok((
        StatusCode::CREATED,
        Json(CustomerAddressMapper::json(saved)),
    ))
}

#[utoipa::path(
    put,
    path = "/customer-addresses/{id}",
    tag = "Customer",
    params(("id" = i64, Path, description = "Customer Address ID")),
    request_body = CustomerAddressJson,
    responses(
        (status = 200, description = "Customer address updated", body = CustomerAddressJson),
        (status = 400, description = "Bad request", body = BadRequestErrorJson),
        (status = 404, description = "Not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(input): Json<CustomerAddressJson>,
) -> HttpResponse<Json<CustomerAddressJson>> {
    let use_case =
        CustomerAddressUseCase::new(CustomerAddressGateway::new(state.conn.as_ref().clone()));
    let existing = use_case
        .find_by_id(id)
        .await
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if !can_read_tenant(&user, existing.tenant_id)
        || tenant_for_write(&user, input.tenant_id.or(existing.tenant_id)) != existing.tenant_id
    {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let updated = CustomerAddressMapper::domain(input);

    let saved = use_case
        .update(id, updated)
        .await
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    Ok(Json(CustomerAddressMapper::json(saved)))
}
