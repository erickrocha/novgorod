use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::cart_json::*;
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::infrastructure::mapper::{CartItemMapper, Mapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::cart_item::CartItemEntityMapper;
use business::domain::enums::Role;
use business::domain::user::User;
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, NotSet, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};
use entity::cart_item_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const CART_ITEM_SORT_FIELDS: &[&str] = &[
    "id",
    "cartId",
    "cart_id",
    "skuId",
    "sku_id",
    "quantity",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/cart-items",
    tag = "Cart",
    responses(
        (status = 200, description = "List of cart items", body = [CartItemJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<CartItemJson>> {
    let mut query = cart_item_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(cart_item_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query.all(state.conn.as_ref()).await.unwrap_or_default();
    let domains = CartItemEntityMapper::from_models(r);
    Json(CartItemMapper::json_vec(domains))
}

#[utoipa::path(
    get,
    path = "/cart-items/paged",
    tag = "Cart",
    params(CartItemPageQuery),
    responses(
        (status = 200, description = "Paged cart items", body = PagedResponse<CartItemJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CartItemPageQuery>,
) -> Json<PagedResponse<CartItemJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), CART_ITEM_SORT_FIELDS, "id");
    let mut query = cart_item_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(cart_item_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(cart_item_entity::Column::TenantId.eq(tid));
    }

    if let Some(cid) = params.cart_id {
        query = query.filter(cart_item_entity::Column::CartId.eq(cid));
    }

    if let Some(sid) = params.sku_id {
        query = query.filter(cart_item_entity::Column::SkuId.eq(sid));
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "cartid" | "cart_id" => cart_item_entity::Column::CartId,
        "skuid" | "sku_id" => cart_item_entity::Column::SkuId,
        "quantity" => cart_item_entity::Column::Quantity,
        "createdat" | "created_at" => cart_item_entity::Column::CreatedAt,
        _ => cart_item_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(cart_item_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(cart_item_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = CartItemEntityMapper::from_models(rows);
    let items = CartItemMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/cart-items/{id}",
    tag = "Cart",
    params(("id" = i64, Path, description = "Cart Item ID")),
    responses(
        (status = 200, description = "Cart item found", body = CartItemJson),
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
) -> HttpResponse<Json<CartItemJson>> {
    let item = cart_item_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
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

    let domain = CartItemEntityMapper::from_model(item);
    Ok(Json(CartItemMapper::json(domain)))
}

#[utoipa::path(
    post,
    path = "/cart-items",
    tag = "Cart",
    request_body = CartItemInputJson,
    responses(
        (status = 201, description = "Cart item created", body = CartItemJson),
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
    Json(input): Json<CartItemInputJson>,
) -> HttpResponse<(StatusCode, Json<CartItemJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.quantity <= 0 || input.unit_price_cents < 0 {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let model = cart_item_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        cart_id: Set(input.cart_id),
        sku_id: Set(input.sku_id),
        quantity: Set(input.quantity),
        unit_price_cents: Set(input.unit_price_cents),
        created_at: NotSet,
        created_by: NotSet,
        updated_at: NotSet,
        updated_by: NotSet,
    };

    let saved = model
        .insert(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = CartItemEntityMapper::from_model(saved);
    Ok((StatusCode::CREATED, Json(CartItemMapper::json(domain))))
}

#[utoipa::path(
    put,
    path = "/cart-items/{id}",
    tag = "Cart",
    params(("id" = i64, Path, description = "Cart Item ID")),
    request_body = CartItemInputJson,
    responses(
        (status = 200, description = "Cart item updated", body = CartItemJson),
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
    Json(input): Json<CartItemInputJson>,
) -> HttpResponse<Json<CartItemJson>> {
    let existing = cart_item_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if !can_read_tenant(&user, existing.tenant_id)
        || tenant_for_write(&user, input.tenant_id.or(existing.tenant_id)) != existing.tenant_id
        || input.quantity <= 0
        || input.unit_price_cents < 0
    {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let mut model = existing.into_active_model();
    model.quantity = Set(input.quantity);
    model.unit_price_cents = Set(input.unit_price_cents);

    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = CartItemEntityMapper::from_model(saved);
    Ok(Json(CartItemMapper::json(domain)))
}
