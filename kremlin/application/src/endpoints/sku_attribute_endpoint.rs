use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::endpoints::json::sku_attribute_json::*;
use crate::infrastructure::mapper::{Mapper, SkuAttributeValueMapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::enums::Role;
use business::domain::sku_attribute_value::SkuAttributeValueEntityMapper;
use business::domain::user::User;
use business::gateway::sku_attribute_value_gateway::SkuAttributeValueGateway;
use business::sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::sku_attribute_value_use_case::SkuAttributeValueUseCase;
use entity::sku_attribute_value_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const SKU_ATTRIBUTE_SORT_FIELDS: &[&str] = &[
    "id",
    "skuId",
    "sku_id",
    "productId",
    "product_id",
    "attributeId",
    "attribute_id",
    "attributeValueId",
    "attribute_value_id",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/sku-attributes",
    tag = "SkuAttribute",
    responses(
        (status = 200, description = "List of SKU attribute assignments", body = [SkuAttributeValueJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<SkuAttributeValueJson>> {
    let usecase = SkuAttributeValueUseCase::new(SkuAttributeValueGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_all().await;
    Json(SkuAttributeValueMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/sku-attributes/paged",
    tag = "SkuAttribute",
    params(SkuAttributeValuePageQuery),
    responses(
        (status = 200, description = "Paged SKU attributes", body = PagedResponse<SkuAttributeValueJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<SkuAttributeValuePageQuery>,
) -> Json<PagedResponse<SkuAttributeValueJson>> {
    let norm =
        NormalizedPagination::new(&params.to_page_query(), SKU_ATTRIBUTE_SORT_FIELDS, "id");
    let mut query = sku_attribute_value_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(sku_attribute_value_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::new(Vec::new(), 0, norm.page, norm.page_size));
        }
    } else if let Some(tenant_id) = params.tenant_id {
        query = query.filter(sku_attribute_value_entity::Column::TenantId.eq(tenant_id));
    }

    if let Some(product_id) = params.product_id {
        query = query.filter(sku_attribute_value_entity::Column::ProductId.eq(product_id));
    }
    if let Some(sku_id) = params.sku_id {
        query = query.filter(sku_attribute_value_entity::Column::SkuId.eq(sku_id));
    }
    if let Some(attribute_id) = params.attribute_id {
        query = query.filter(sku_attribute_value_entity::Column::AttributeId.eq(attribute_id));
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "skuid" | "sku_id" => sku_attribute_value_entity::Column::SkuId,
        "productid" | "product_id" => sku_attribute_value_entity::Column::ProductId,
        "attributeid" | "attribute_id" => sku_attribute_value_entity::Column::AttributeId,
        "attributevalueid" | "attribute_value_id" => sku_attribute_value_entity::Column::AttributeValueId,
        "createdat" | "created_at" => sku_attribute_value_entity::Column::CreatedAt,
        _ => sku_attribute_value_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(sku_attribute_value_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(sku_attribute_value_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = SkuAttributeValueEntityMapper::from_models(rows);
    let items = SkuAttributeValueMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/sku-attributes/{id}",
    tag = "SkuAttribute",
    params(("id" = i64, Path, description = "SKU Attribute ID")),
    responses(
        (status = 200, description = "SKU attribute assignment found", body = SkuAttributeValueJson),
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
) -> HttpResponse<Json<SkuAttributeValueJson>> {
    let usecase = SkuAttributeValueUseCase::new(SkuAttributeValueGateway::new(state.conn.as_ref().clone()));
    let item = usecase.find_by_id(id).await.ok_or(ExceptionResponse::NotFound(
        locale,
        ErrorKey::InvalidParameterValue,
    ))?;

    if !can_read_tenant(&current_user, item.tenant_id) {
        return Err(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    Ok(Json(SkuAttributeValueMapper::json(item)))
}

#[utoipa::path(
    get,
    path = "/sku-attributes/by-sku/{sku_id}",
    tag = "SkuAttribute",
    params(("sku_id" = i64, Path, description = "SKU ID")),
    responses(
        (status = 200, description = "List of SKU attribute assignments for SKU", body = [SkuAttributeValueJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn by_sku(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
    Path(sku_id): Path<i64>,
) -> Json<Vec<SkuAttributeValueJson>> {
    let usecase = SkuAttributeValueUseCase::new(SkuAttributeValueGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_by_sku_id(sku_id).await;
    Json(SkuAttributeValueMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/sku-attributes/by-product/{product_id}",
    tag = "SkuAttribute",
    params(("product_id" = i64, Path, description = "Product ID")),
    responses(
        (status = 200, description = "List of SKU attribute assignments for product", body = [SkuAttributeValueJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn by_product(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
    Path(product_id): Path<i64>,
) -> Json<Vec<SkuAttributeValueJson>> {
    let usecase = SkuAttributeValueUseCase::new(SkuAttributeValueGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_by_product_id(product_id).await;
    Json(SkuAttributeValueMapper::json_vec(items))
}

#[utoipa::path(
    post,
    path = "/sku-attributes",
    tag = "SkuAttribute",
    request_body = SkuAttributeValueInputJson,
    responses(
        (status = 201, description = "SKU attribute assignment created", body = SkuAttributeValueJson),
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
    Json(input): Json<SkuAttributeValueInputJson>,
) -> HttpResponse<(StatusCode, Json<SkuAttributeValueJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.product_id <= 0
        || input.sku_id <= 0
        || input.product_attribute_id <= 0
        || input.attribute_id <= 0
        || input.attribute_value_id <= 0
    {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let sav = SkuAttributeValueMapper::domain(SkuAttributeValueJson {
        id: 0,
        uuid: String::new(),
        tenant_id: Some(tenant_id),
        product_id: input.product_id,
        sku_id: input.sku_id,
        product_attribute_id: input.product_attribute_id,
        attribute_id: input.attribute_id,
        attribute_value_id: input.attribute_value_id,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    });

    let usecase = SkuAttributeValueUseCase::new(SkuAttributeValueGateway::new(state.conn.as_ref().clone()));
    let saved = usecase
        .create(sav)
        .await
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    Ok((StatusCode::CREATED, Json(SkuAttributeValueMapper::json(saved))))
}

#[utoipa::path(
    put,
    path = "/sku-attributes/{id}",
    tag = "SkuAttribute",
    params(("id" = i64, Path, description = "SKU Attribute ID")),
    request_body = SkuAttributeValueInputJson,
    responses(
        (status = 200, description = "SKU attribute updated", body = SkuAttributeValueJson),
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
    Json(input): Json<SkuAttributeValueInputJson>,
) -> HttpResponse<Json<SkuAttributeValueJson>> {
    let usecase = SkuAttributeValueUseCase::new(SkuAttributeValueGateway::new(state.conn.as_ref().clone()));
    let existing = usecase
        .find_by_id(id)
        .await
        .ok_or(ExceptionResponse::NotFound(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    if !can_read_tenant(&user, existing.tenant_id)
        || tenant_for_write(&user, input.tenant_id.or(existing.tenant_id)) != existing.tenant_id
        || input.product_id <= 0
        || input.sku_id <= 0
        || input.product_attribute_id <= 0
        || input.attribute_id <= 0
        || input.attribute_value_id <= 0
    {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let sav = SkuAttributeValueMapper::domain(SkuAttributeValueJson {
        id,
        uuid: existing.uuid.unwrap_or_default(),
        tenant_id: existing.tenant_id,
        product_id: input.product_id,
        sku_id: input.sku_id,
        product_attribute_id: input.product_attribute_id,
        attribute_id: input.attribute_id,
        attribute_value_id: input.attribute_value_id,
        created_at: existing.created_at,
        created_by: existing.created_by,
        updated_at: existing.updated_at,
        updated_by: existing.updated_by,
    });

    let saved = usecase
        .update(id, sav)
        .await
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    Ok(Json(SkuAttributeValueMapper::json(saved)))
}

#[utoipa::path(
    delete,
    path = "/sku-attributes/{id}",
    tag = "SkuAttribute",
    params(("id" = i64, Path, description = "SKU Attribute ID")),
    responses(
        (status = 204, description = "SKU attribute deleted"),
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
    let usecase = SkuAttributeValueUseCase::new(SkuAttributeValueGateway::new(state.conn.as_ref().clone()));
    let existing = usecase
        .find_by_id(id)
        .await
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

    usecase
        .delete_by_id(id)
        .await
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    Ok(StatusCode::NO_CONTENT)
}
