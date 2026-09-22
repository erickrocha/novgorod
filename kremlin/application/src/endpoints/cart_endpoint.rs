use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::cart_json::*;
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::infrastructure::mapper::{CartMapper, Mapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::cart::{Cart, CartEntityMapper};
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::cart_gateway::CartGateway;
use business::sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::cart_use_case::CartUseCase;
use entity::cart_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const CART_SORT_FIELDS: &[&str] = &[
    "id",
    "status",
    "customerId",
    "customer_id",
    "expiresAt",
    "expires_at",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/carts",
    tag = "Cart",
    responses(
        (status = 200, description = "List of carts", body = [CartJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<CartJson>> {
    let usecase = CartUseCase::new(CartGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_all().await;
    Json(CartMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/carts/paged",
    tag = "Cart",
    params(CartPageQuery),
    responses(
        (status = 200, description = "Paged carts", body = PagedResponse<CartJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CartPageQuery>,
) -> Json<PagedResponse<CartJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), CART_SORT_FIELDS, "id");
    let mut query = cart_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(cart_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(cart_entity::Column::TenantId.eq(tid));
    }

    if let Some(ref st) = params.status {
        query = query.filter(cart_entity::Column::Status.eq(st));
    }

    if let Some(cid) = params.customer_id {
        query = query.filter(cart_entity::Column::CustomerId.eq(cid));
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "status" => cart_entity::Column::Status,
        "customerid" | "customer_id" => cart_entity::Column::CustomerId,
        "expiresat" | "expires_at" => cart_entity::Column::ExpiresAt,
        "createdat" | "created_at" => cart_entity::Column::CreatedAt,
        _ => cart_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(cart_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(cart_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = CartEntityMapper::from_models(rows);
    let items = CartMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/carts/{id}",
    tag = "Cart",
    params(("id" = i64, Path, description = "Cart ID")),
    responses(
        (status = 200, description = "Cart found", body = CartJson),
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
) -> HttpResponse<Json<CartJson>> {
    let usecase = CartUseCase::new(CartGateway::new(state.conn.as_ref().clone()));
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

    Ok(Json(CartMapper::json(item)))
}

#[utoipa::path(
    post,
    path = "/carts",
    tag = "Cart",
    request_body = CartInputJson,
    responses(
        (status = 201, description = "Cart created", body = CartJson),
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
    Extension(user): Extension<User>,
    Json(input): Json<CartInputJson>,
) -> HttpResponse<(StatusCode, Json<CartJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    let status = input
        .status
        .unwrap_or_else(|| "active".to_string())
        .to_lowercase();

    let domain = Cart {
        id: None,
        uuid: None,
        tenant_id: Some(tenant_id),
        customer_id: input.customer_id,
        status,
        expires_at: input.expires_at,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    };

    let usecase = CartUseCase::new(CartGateway::new(state.conn.as_ref().clone()));
    let saved = usecase
        .create(domain)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok((StatusCode::CREATED, Json(CartMapper::json(saved))))
}

#[utoipa::path(
    put,
    path = "/carts/{id}",
    tag = "Cart",
    params(("id" = i64, Path, description = "Cart ID")),
    request_body = CartInputJson,
    responses(
        (status = 200, description = "Cart updated", body = CartJson),
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
    Json(input): Json<CartInputJson>,
) -> HttpResponse<Json<CartJson>> {
    let usecase = CartUseCase::new(CartGateway::new(state.conn.as_ref().clone()));
    let existing = usecase
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

    let mut updated = existing;
    if let Some(cid) = input.customer_id {
        updated.customer_id = Some(cid);
    }
    if let Some(st) = input.status {
        updated.status = st.to_lowercase();
    }
    if input.expires_at.is_some() {
        updated.expires_at = input.expires_at;
    }

    let saved = usecase
        .update(id, updated)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok(Json(CartMapper::json(saved)))
}
