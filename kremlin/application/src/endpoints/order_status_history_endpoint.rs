use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::endpoints::json::orders_json::*;
use crate::infrastructure::mapper::{Mapper, OrderStatusHistoryMapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::enums::Role;
use business::domain::order_status_history::{OrderStatusHistory, OrderStatusHistoryEntityMapper};
use business::domain::user::User;
use business::gateway::order_status_history_gateway::OrderStatusHistoryGateway;
use business::sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::order_status_history_use_case::OrderStatusHistoryUseCase;
use entity::order_status_history_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser | Role::Customer => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const ORDER_STATUS_HISTORY_SORT_FIELDS: &[&str] = &[
    "id",
    "orderId",
    "order_id",
    "fromStatus",
    "from_status",
    "toStatus",
    "to_status",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/order-status-histories",
    tag = "Orders",
    responses(
        (status = 200, description = "List of order status histories", body = [OrderStatusHistoryJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<OrderStatusHistoryJson>> {
    let usecase = OrderStatusHistoryUseCase::new(OrderStatusHistoryGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_all().await;
    Json(OrderStatusHistoryMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/order-status-histories/paged",
    tag = "Orders",
    params(OrderStatusHistoryPageQuery),
    responses(
        (status = 200, description = "Paged order status histories", body = PagedResponse<OrderStatusHistoryJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<OrderStatusHistoryPageQuery>,
) -> Json<PagedResponse<OrderStatusHistoryJson>> {
    let norm = NormalizedPagination::new(
        &params.to_page_query(),
        ORDER_STATUS_HISTORY_SORT_FIELDS,
        "id",
    );
    let mut query = order_status_history_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(order_status_history_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(order_status_history_entity::Column::TenantId.eq(tid));
    }

    if let Some(oid) = params.order_id {
        query = query.filter(order_status_history_entity::Column::OrderId.eq(oid));
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "orderid" | "order_id" => order_status_history_entity::Column::OrderId,
        "fromstatus" | "from_status" => order_status_history_entity::Column::FromStatus,
        "tostatus" | "to_status" => order_status_history_entity::Column::ToStatus,
        "createdat" | "created_at" => order_status_history_entity::Column::CreatedAt,
        _ => order_status_history_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(order_status_history_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(order_status_history_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = OrderStatusHistoryEntityMapper::from_models(rows);
    let items = OrderStatusHistoryMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/order-status-histories/{id}",
    tag = "Orders",
    params(("id" = i64, Path, description = "Order Status History ID")),
    responses(
        (status = 200, description = "Order status history found", body = OrderStatusHistoryJson),
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
) -> HttpResponse<Json<OrderStatusHistoryJson>> {
    let usecase = OrderStatusHistoryUseCase::new(OrderStatusHistoryGateway::new(state.conn.as_ref().clone()));
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

    Ok(Json(OrderStatusHistoryMapper::json(item)))
}

#[utoipa::path(
    post,
    path = "/order-status-histories",
    tag = "Orders",
    request_body = OrderStatusHistoryInputJson,
    responses(
        (status = 201, description = "Order status history created", body = OrderStatusHistoryJson),
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
    Json(input): Json<OrderStatusHistoryInputJson>,
) -> HttpResponse<(StatusCode, Json<OrderStatusHistoryJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.to_status.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let domain = OrderStatusHistory {
        id: None,
        uuid: None,
        tenant_id: Some(tenant_id),
        order_id: input.order_id,
        from_status: input.from_status,
        to_status: input.to_status.trim().to_string(),
        actor_type: input.actor_type.trim().to_string(),
        actor_id: input.actor_id,
        note: input.note,
        created_at: None,
        created_by: None,
    };

    let usecase = OrderStatusHistoryUseCase::new(OrderStatusHistoryGateway::new(state.conn.as_ref().clone()));
    let saved = usecase
        .create(domain)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok((
        StatusCode::CREATED,
        Json(OrderStatusHistoryMapper::json(saved)),
    ))
}
