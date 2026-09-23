use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::coupon_json::*;
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::infrastructure::mapper::{CouponMapper, Mapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::coupon::{Coupon, CouponEntityMapper};
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::coupon_gateway::CouponGateway;
use business::sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::coupon_use_case::CouponUseCase;
use entity::coupon_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser | Role::Customer => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const COUPON_SORT_FIELDS: &[&str] = &[
    "id",
    "code",
    "couponType",
    "coupon_type",
    "value",
    "active",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/coupons",
    tag = "Coupon",
    responses(
        (status = 200, description = "List of coupons", body = [CouponJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<CouponJson>> {
    let usecase = CouponUseCase::new(CouponGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_all().await;
    Json(CouponMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/coupons/paged",
    tag = "Coupon",
    params(CouponPageQuery),
    responses(
        (status = 200, description = "Paged coupons", body = PagedResponse<CouponJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CouponPageQuery>,
) -> Json<PagedResponse<CouponJson>> {
    let norm = NormalizedPagination::new(&params.to_page_query(), COUPON_SORT_FIELDS, "code");
    let mut query = coupon_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(coupon_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(coupon_entity::Column::TenantId.eq(tid));
    }

    if let Some(active) = params.active {
        query = query.filter(coupon_entity::Column::Active.eq(active));
    }

    if let Some(ref q) = norm.q {
        let pattern = format!("%{}%", q);
        query = query.filter(
            Condition::any()
                .add(coupon_entity::Column::Code.like(&pattern))
                .add(coupon_entity::Column::CouponType.like(&pattern)),
        );
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "coupontype" | "coupon_type" => coupon_entity::Column::CouponType,
        "value" => coupon_entity::Column::Value,
        "active" => coupon_entity::Column::Active,
        "createdat" | "created_at" => coupon_entity::Column::CreatedAt,
        "id" => coupon_entity::Column::Id,
        _ => coupon_entity::Column::Code,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(coupon_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(coupon_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = CouponEntityMapper::from_models(rows);
    let items = CouponMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/coupons/{id}",
    tag = "Coupon",
    params(("id" = i64, Path, description = "Coupon ID")),
    responses(
        (status = 200, description = "Coupon found", body = CouponJson),
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
) -> HttpResponse<Json<CouponJson>> {
    let usecase = CouponUseCase::new(CouponGateway::new(state.conn.as_ref().clone()));
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

    Ok(Json(CouponMapper::json(item)))
}

#[utoipa::path(
    post,
    path = "/coupons",
    tag = "Coupon",
    request_body = CouponInputJson,
    responses(
        (status = 201, description = "Coupon created", body = CouponJson),
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
    Json(input): Json<CouponInputJson>,
) -> HttpResponse<(StatusCode, Json<CouponJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.code.trim().is_empty() || input.coupon_type.trim().is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let domain = Coupon {
        id: None,
        uuid: None,
        tenant_id: Some(tenant_id),
        code: input.code.trim().to_uppercase(),
        campaign_id: input.campaign_id,
        coupon_type: input.coupon_type.trim().to_string(),
        value: input.value,
        min_order_cents: input.min_order_cents,
        max_uses: input.max_uses,
        max_uses_per_customer: input.max_uses_per_customer,
        starts_at: input.starts_at,
        expires_at: input.expires_at,
        active: input.active.unwrap_or(true),
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    };

    let usecase = CouponUseCase::new(CouponGateway::new(state.conn.as_ref().clone()));
    let saved = usecase
        .create(domain)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok((StatusCode::CREATED, Json(CouponMapper::json(saved))))
}

#[utoipa::path(
    put,
    path = "/coupons/{id}",
    tag = "Coupon",
    params(("id" = i64, Path, description = "Coupon ID")),
    request_body = CouponInputJson,
    responses(
        (status = 200, description = "Coupon updated", body = CouponJson),
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
    Json(input): Json<CouponInputJson>,
) -> HttpResponse<Json<CouponJson>> {
    let usecase = CouponUseCase::new(CouponGateway::new(state.conn.as_ref().clone()));
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
    updated.code = input.code.trim().to_uppercase();
    updated.campaign_id = input.campaign_id;
    updated.coupon_type = input.coupon_type.trim().to_string();
    updated.value = input.value;
    updated.min_order_cents = input.min_order_cents;
    updated.max_uses = input.max_uses;
    updated.max_uses_per_customer = input.max_uses_per_customer;
    updated.starts_at = input.starts_at;
    updated.expires_at = input.expires_at;
    if let Some(act) = input.active {
        updated.active = act;
    }

    let saved = usecase
        .update(id, updated)
        .await
        .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok(Json(CouponMapper::json(saved)))
}
