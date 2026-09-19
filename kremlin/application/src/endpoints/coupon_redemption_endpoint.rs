use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::coupon_json::*;
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::infrastructure::mapper::{CouponRedemptionMapper, Mapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::coupon_redemption::CouponRedemptionEntityMapper;
use business::domain::enums::Role;
use business::domain::user::User;
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use entity::coupon_redemption_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const COUPON_REDEMPTION_SORT_FIELDS: &[&str] = &[
    "id",
    "couponId",
    "coupon_id",
    "orderId",
    "order_id",
    "customerId",
    "customer_id",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/coupon-redemptions",
    tag = "Coupon",
    responses(
        (status = 200, description = "List of coupon redemptions", body = [CouponRedemptionJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<CouponRedemptionJson>> {
    let mut query = coupon_redemption_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(coupon_redemption_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query.all(state.conn.as_ref()).await.unwrap_or_default();
    let domains = CouponRedemptionEntityMapper::from_models(r);
    Json(CouponRedemptionMapper::json_vec(domains))
}

#[utoipa::path(
    get,
    path = "/coupon-redemptions/paged",
    tag = "Coupon",
    params(CouponRedemptionPageQuery),
    responses(
        (status = 200, description = "Paged coupon redemptions", body = PagedResponse<CouponRedemptionJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<CouponRedemptionPageQuery>,
) -> Json<PagedResponse<CouponRedemptionJson>> {
    let norm =
        NormalizedPagination::new(&params.to_page_query(), COUPON_REDEMPTION_SORT_FIELDS, "id");
    let mut query = coupon_redemption_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(coupon_redemption_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::empty(norm.page, norm.page_size));
        }
    } else if let Some(tid) = params.tenant_id {
        query = query.filter(coupon_redemption_entity::Column::TenantId.eq(tid));
    }

    if let Some(cid) = params.coupon_id {
        query = query.filter(coupon_redemption_entity::Column::CouponId.eq(cid));
    }

    if let Some(oid) = params.order_id {
        query = query.filter(coupon_redemption_entity::Column::OrderId.eq(oid));
    }

    if let Some(cuid) = params.customer_id {
        query = query.filter(coupon_redemption_entity::Column::CustomerId.eq(cuid));
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "couponid" | "coupon_id" => coupon_redemption_entity::Column::CouponId,
        "orderid" | "order_id" => coupon_redemption_entity::Column::OrderId,
        "customerid" | "customer_id" => coupon_redemption_entity::Column::CustomerId,
        "createdat" | "created_at" => coupon_redemption_entity::Column::CreatedAt,
        _ => coupon_redemption_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(coupon_redemption_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(coupon_redemption_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = CouponRedemptionEntityMapper::from_models(rows);
    let items = CouponRedemptionMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/coupon-redemptions/{id}",
    tag = "Coupon",
    params(("id" = i64, Path, description = "Coupon Redemption ID")),
    responses(
        (status = 200, description = "Coupon redemption found", body = CouponRedemptionJson),
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
) -> HttpResponse<Json<CouponRedemptionJson>> {
    let item = coupon_redemption_entity::Entity::find_by_id(id)
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

    let domain = CouponRedemptionEntityMapper::from_model(item);
    Ok(Json(CouponRedemptionMapper::json(domain)))
}

#[utoipa::path(
    post,
    path = "/coupon-redemptions",
    tag = "Coupon",
    request_body = CouponRedemptionInputJson,
    responses(
        (status = 201, description = "Coupon redemption created", body = CouponRedemptionJson),
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
    Json(input): Json<CouponRedemptionInputJson>,
) -> HttpResponse<(StatusCode, Json<CouponRedemptionJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    let model = coupon_redemption_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        coupon_id: Set(input.coupon_id),
        order_id: Set(input.order_id),
        customer_id: Set(input.customer_id),
        created_at: NotSet,
        created_by: NotSet,
    };

    let saved = model
        .insert(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = CouponRedemptionEntityMapper::from_model(saved);
    Ok((
        StatusCode::CREATED,
        Json(CouponRedemptionMapper::json(domain)),
    ))
}
