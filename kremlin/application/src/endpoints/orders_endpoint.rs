use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::endpoints::json::orders_json::*;
use crate::infrastructure::mapper::{Mapper, OrdersMapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::enums::Role;
use business::domain::orders::OrdersEntityMapper;
use business::domain::user::User;
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, NotSet,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use entity::orders_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const ORDERS_SORT_FIELDS: &[&str] = &[
    "id",
    "number",
    "status",
    "paymentStatus",
    "payment_status",
    "totalCents",
    "total_cents",
    "placedAt",
    "placed_at",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/orders",
    tag = "Orders",
    responses(
        (status = 200, description = "List of orders", body = [OrdersJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<OrdersJson>> {
    let mut query = orders_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(orders_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query
        .order_by_desc(orders_entity::Column::Id)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();
    let domains = OrdersEntityMapper::from_models(r);
    Json(OrdersMapper::json_vec(domains))
}

#[utoipa::path(
    get,
    path = "/orders/paged",
    tag = "Orders",
    params(OrdersPageQuery),
    responses(
        (status = 200, description = "Paged orders", body = PagedResponse<OrdersJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<OrdersPageQuery>,
) -> Json<PagedResponse<OrdersJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), ORDERS_SORT_FIELDS, "id");
    let mut query = orders_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(orders_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(orders_entity::Column::TenantId.eq(tid));
    }

    if let Some(ref st) = params.status {
        query = query.filter(orders_entity::Column::Status.eq(st));
    }

    if let Some(cid) = params.customer_id {
        query = query.filter(orders_entity::Column::CustomerId.eq(cid));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(orders_entity::Column::Number.like(&pattern))
                .add(orders_entity::Column::ShipRecipient.like(&pattern))
                .add(orders_entity::Column::Status.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "number" => orders_entity::Column::Number,
        "status" => orders_entity::Column::Status,
        "paymentstatus" | "payment_status" => orders_entity::Column::PaymentStatus,
        "totalcents" | "total_cents" => orders_entity::Column::TotalCents,
        "placedat" | "placed_at" => orders_entity::Column::PlacedAt,
        "createdat" | "created_at" => orders_entity::Column::CreatedAt,
        _ => orders_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(orders_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(orders_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = OrdersEntityMapper::from_models(rows);
    let items = OrdersMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/orders/{id}",
    tag = "Orders",
    params(("id" = i64, Path, description = "Order ID")),
    responses(
        (status = 200, description = "Order found", body = OrdersJson),
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
) -> HttpResponse<Json<OrdersJson>> {
    let item = orders_entity::Entity::find_by_id(id)
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

    let domain = OrdersEntityMapper::from_model(item);
    Ok(Json(OrdersMapper::json(domain)))
}

#[utoipa::path(
    post,
    path = "/orders",
    tag = "Orders",
    request_body = OrdersInputJson,
    responses(
        (status = 201, description = "Order created", body = OrdersJson),
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
    Json(input): Json<OrdersInputJson>,
) -> HttpResponse<(StatusCode, Json<OrdersJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    let number = input.number.unwrap_or_else(|| {
        format!("ORD-{}", chrono::Utc::now().timestamp_millis())
    });

    let now = chrono::Utc::now().naive_utc();
    let model = orders_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        number: Set(number),
        customer_id: Set(input.customer_id),
        status: Set(input.status.unwrap_or_else(|| "pending".to_string())),
        payment_status: Set(input.payment_status.unwrap_or_else(|| "pending".to_string())),
        subtotal_cents: Set(input.subtotal_cents),
        discount_cents: Set(input.discount_cents.unwrap_or(0)),
        shipping_cents: Set(input.shipping_cents.unwrap_or(0)),
        tax_total_cents: Set(input.tax_total_cents.unwrap_or(0)),
        total_cents: Set(input.total_cents),
        coupon_id: Set(input.coupon_id),
        coupon_code: Set(input.coupon_code),
        ship_recipient: Set(input.ship_recipient),
        ship_cep: Set(input.ship_cep),
        ship_logradouro: Set(input.ship_logradouro),
        ship_numero: Set(input.ship_numero),
        ship_complemento: Set(input.ship_complemento),
        ship_bairro: Set(input.ship_bairro),
        ship_cidade: Set(input.ship_cidade),
        ship_uf: Set(input.ship_uf),
        placed_at: Set(now),
        created_at: NotSet,
        created_by: NotSet,
        updated_at: NotSet,
        updated_by: NotSet,
    };

    let saved = model
        .insert(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = OrdersEntityMapper::from_model(saved);
    Ok((StatusCode::CREATED, Json(OrdersMapper::json(domain))))
}

#[utoipa::path(
    put,
    path = "/orders/{id}",
    tag = "Orders",
    params(("id" = i64, Path, description = "Order ID")),
    request_body = OrdersInputJson,
    responses(
        (status = 200, description = "Order updated", body = OrdersJson),
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
    Json(input): Json<OrdersInputJson>,
) -> HttpResponse<Json<OrdersJson>> {
    let existing = orders_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
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

    let mut model = existing.into_active_model();
    if let Some(st) = input.status {
        model.status = Set(st);
    }
    if let Some(ps) = input.payment_status {
        model.payment_status = Set(ps);
    }

    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = OrdersEntityMapper::from_model(saved);
    Ok(Json(OrdersMapper::json(domain)))
}
