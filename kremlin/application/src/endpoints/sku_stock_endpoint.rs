use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::endpoints::json::sku_stock_json::*;
use crate::infrastructure::mapper::{Mapper, SkuStockMapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::enums::Role;
use business::domain::sku_stock::SkuStockEntityMapper;
use business::domain::user::User;
use business::sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, NotSet, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};
use entity::sku_stock_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const SKU_STOCK_SORT_FIELDS: &[&str] = &[
    "id",
    "skuId",
    "sku_id",
    "quantity",
    "reserved",
    "createdAt",
    "created_at",
    "updatedAt",
    "updated_at",
];

#[utoipa::path(
    get,
    path = "/sku-stocks",
    tag = "SkuStock",
    responses(
        (status = 200, description = "List of SKU inventory records", body = [SkuStockJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Json<Vec<SkuStockJson>> {
    let mut query = sku_stock_entity::Entity::find();
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(sku_stock_entity::Column::TenantId.eq(id));
        } else {
            return Json(Vec::new());
        }
    }
    let r = query
        .order_by_asc(sku_stock_entity::Column::Id)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();
    let domains = SkuStockEntityMapper::from_models(r);
    Json(SkuStockMapper::json_vec(domains))
}

#[utoipa::path(
    get,
    path = "/sku-stocks/paged",
    tag = "SkuStock",
    params(SkuStockPageQuery),
    responses(
        (status = 200, description = "Paged SKU stock records", body = PagedResponse<SkuStockJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<SkuStockPageQuery>,
) -> Json<PagedResponse<SkuStockJson>> {
    let norm =
        NormalizedPagination::new(&params.to_page_query(), SKU_STOCK_SORT_FIELDS, "id");
    let mut query = sku_stock_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(sku_stock_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::new(Vec::new(), 0, norm.page, norm.page_size));
        }
    } else if let Some(tenant_id) = params.tenant_id {
        query = query.filter(sku_stock_entity::Column::TenantId.eq(tenant_id));
    }

    if let Some(sku_id) = params.sku_id {
        query = query.filter(sku_stock_entity::Column::SkuId.eq(sku_id));
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "skuid" | "sku_id" => sku_stock_entity::Column::SkuId,
        "quantity" => sku_stock_entity::Column::Quantity,
        "reserved" => sku_stock_entity::Column::Reserved,
        "createdat" | "created_at" => sku_stock_entity::Column::CreatedAt,
        "updatedat" | "updated_at" => sku_stock_entity::Column::UpdatedAt,
        _ => sku_stock_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(sku_stock_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(sku_stock_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = SkuStockEntityMapper::from_models(rows);
    let items = SkuStockMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/sku-stocks/{id}",
    tag = "SkuStock",
    params(("id" = i64, Path, description = "SKU Stock ID")),
    responses(
        (status = 200, description = "SKU stock record found", body = SkuStockJson),
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
) -> HttpResponse<Json<SkuStockJson>> {
    let item = sku_stock_entity::Entity::find_by_id(id)
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

    let domain = SkuStockEntityMapper::from_model(item);
    Ok(Json(SkuStockMapper::json(domain)))
}

#[utoipa::path(
    get,
    path = "/sku-stocks/by-sku/{sku_id}",
    tag = "SkuStock",
    params(("sku_id" = i64, Path, description = "SKU ID")),
    responses(
        (status = 200, description = "SKU stock record for SKU", body = SkuStockJson),
        (status = 404, description = "Not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn by_sku(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(current_user): Extension<User>,
    Path(sku_id): Path<i64>,
) -> HttpResponse<Json<SkuStockJson>> {
    let mut query = sku_stock_entity::Entity::find()
        .filter(sku_stock_entity::Column::SkuId.eq(sku_id));
    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(sku_stock_entity::Column::TenantId.eq(id));
        } else {
            return Err(ExceptionResponse::NotFound(
                locale,
                ErrorKey::InvalidParameterValue,
            ));
        }
    }
    let item = query
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    let domain = SkuStockEntityMapper::from_model(item);
    Ok(Json(SkuStockMapper::json(domain)))
}

#[utoipa::path(
    post,
    path = "/sku-stocks",
    tag = "SkuStock",
    request_body = SkuStockInputJson,
    responses(
        (status = 201, description = "SKU stock record created", body = SkuStockJson),
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
    Json(input): Json<SkuStockInputJson>,
) -> HttpResponse<(StatusCode, Json<SkuStockJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.sku_id <= 0 || input.quantity < 0 {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let model = sku_stock_entity::ActiveModel {
        id: NotSet,
        uuid: NotSet,
        tenant_id: Set(Some(tenant_id)),
        sku_id: Set(input.sku_id),
        quantity: Set(input.quantity),
        reserved: Set(input.reserved.unwrap_or(0)),
        created_at: NotSet,
        created_by: NotSet,
        updated_at: NotSet,
        updated_by: NotSet,
    };

    let saved = model
        .insert(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = SkuStockEntityMapper::from_model(saved);
    Ok((StatusCode::CREATED, Json(SkuStockMapper::json(domain))))
}

#[utoipa::path(
    put,
    path = "/sku-stocks/{id}",
    tag = "SkuStock",
    params(("id" = i64, Path, description = "SKU Stock ID")),
    request_body = SkuStockInputJson,
    responses(
        (status = 200, description = "SKU stock updated", body = SkuStockJson),
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
    Json(input): Json<SkuStockInputJson>,
) -> HttpResponse<Json<SkuStockJson>> {
    let existing = sku_stock_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if !can_read_tenant(&user, existing.tenant_id)
        || tenant_for_write(&user, input.tenant_id.or(existing.tenant_id)) != existing.tenant_id
        || input.sku_id <= 0
        || input.quantity < 0
    {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let mut model = existing.into_active_model();
    model.sku_id = Set(input.sku_id);
    model.quantity = Set(input.quantity);
    if let Some(r) = input.reserved {
        model.reserved = Set(r);
    }

    let saved = model
        .update(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let domain = SkuStockEntityMapper::from_model(saved);
    Ok(Json(SkuStockMapper::json(domain)))
}

#[utoipa::path(
    delete,
    path = "/sku-stocks/{id}",
    tag = "SkuStock",
    params(("id" = i64, Path, description = "SKU Stock ID")),
    responses(
        (status = 204, description = "SKU stock record deleted"),
        (status = 404, description = "Not found", body = NotFoundErrorJson),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
) -> HttpResponse<StatusCode> {
    let existing = sku_stock_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if !can_read_tenant(&user, existing.tenant_id)
        || tenant_for_write(&user, existing.tenant_id) != existing.tenant_id
    {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    sku_stock_entity::Entity::delete_by_id(id)
        .exec(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok(StatusCode::NO_CONTENT)
}
