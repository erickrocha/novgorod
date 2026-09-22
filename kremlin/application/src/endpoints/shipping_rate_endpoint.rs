use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::endpoints::json::shipping_rate_json::*;
use crate::infrastructure::mapper::{Mapper, ShippingRateMapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::enums::Role;
use business::domain::shipping_rate::{ShippingRate, ShippingRateEntityMapper};
use business::domain::user::User;
use business::gateway::shipping_rate_gateway::ShippingRateGateway;
use business::sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::shipping_rate_use_case::ShippingRateUseCase;
use entity::shipping_rate_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const SHIPPING_RATE_SORT_FIELDS: &[&str] = &[
    "id",
    "uf",
    "priceCents",
    "price_cents",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/shipping-rates",
    tag = "ShippingRate",
    responses(
        (status = 200, description = "List of shipping rates", body = [ShippingRateJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<ShippingRateJson>> {
    let usecase = ShippingRateUseCase::new(ShippingRateGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_all().await;
    Json(ShippingRateMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/shipping-rates/paged",
    tag = "ShippingRate",
    params(ShippingRatePageQuery),
    responses(
        (status = 200, description = "Paged shipping rates", body = PagedResponse<ShippingRateJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<ShippingRatePageQuery>,
) -> Json<PagedResponse<ShippingRateJson>> {
    let norm =
        NormalizedPagination::new(&params.to_page_query(), SHIPPING_RATE_SORT_FIELDS, "uf");
    let mut query = shipping_rate_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(shipping_rate_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(shipping_rate_entity::Column::TenantId.eq(tid));
    }

    if let Some(ref uf) = params.uf {
        query = query.filter(shipping_rate_entity::Column::Uf.eq(uf));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any().add(shipping_rate_entity::Column::Uf.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "pricecents" | "price_cents" => shipping_rate_entity::Column::PriceCents,
        "createdat" | "created_at" => shipping_rate_entity::Column::CreatedAt,
        "id" => shipping_rate_entity::Column::Id,
        _ => shipping_rate_entity::Column::Uf,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(shipping_rate_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(shipping_rate_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = ShippingRateEntityMapper::from_models(rows);
    let items = ShippingRateMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/shipping-rates/{id}",
    tag = "ShippingRate",
    params(("id" = i64, Path, description = "Shipping Rate ID")),
    responses(
        (status = 200, description = "Shipping rate found", body = ShippingRateJson),
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
) -> HttpResponse<Json<ShippingRateJson>> {
    let usecase = ShippingRateUseCase::new(ShippingRateGateway::new(state.conn.as_ref().clone()));
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

    Ok(Json(ShippingRateMapper::json(item)))
}

#[utoipa::path(
    post,
    path = "/shipping-rates",
    tag = "ShippingRate",
    request_body = ShippingRateInputJson,
    responses(
        (status = 201, description = "Shipping rate created", body = ShippingRateJson),
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
    Json(input): Json<ShippingRateInputJson>,
) -> HttpResponse<(StatusCode, Json<ShippingRateJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.uf.trim().is_empty() || input.price_cents < 0 {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let domain = ShippingRate {
        id: None,
        uuid: None,
        tenant_id: Some(tenant_id),
        uf: input.uf.trim().to_uppercase(),
        price_cents: input.price_cents,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    };

    let usecase = ShippingRateUseCase::new(ShippingRateGateway::new(state.conn.as_ref().clone()));
    let saved = usecase
        .create(domain)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok((StatusCode::CREATED, Json(ShippingRateMapper::json(saved))))
}

#[utoipa::path(
    put,
    path = "/shipping-rates/{id}",
    tag = "ShippingRate",
    params(("id" = i64, Path, description = "Shipping Rate ID")),
    request_body = ShippingRateInputJson,
    responses(
        (status = 200, description = "Shipping rate updated", body = ShippingRateJson),
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
    Json(input): Json<ShippingRateInputJson>,
) -> HttpResponse<Json<ShippingRateJson>> {
    let usecase = ShippingRateUseCase::new(ShippingRateGateway::new(state.conn.as_ref().clone()));
    let existing = usecase
        .find_by_id(id)
        .await
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if !can_read_tenant(&user, existing.tenant_id)
        || tenant_for_write(&user, input.tenant_id.or(existing.tenant_id)) != existing.tenant_id
        || input.price_cents < 0
    {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    if input.uf.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let mut updated = existing;
    updated.uf = input.uf.trim().to_uppercase();
    updated.price_cents = input.price_cents;

    let saved = usecase
        .update(id, updated)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok(Json(ShippingRateMapper::json(saved)))
}
