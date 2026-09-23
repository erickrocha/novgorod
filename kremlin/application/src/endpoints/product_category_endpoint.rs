use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::commons::pagination::{NormalizedPagination, PagedResponse};
use crate::endpoints::json::error_response_json::{
    BadRequestErrorJson, ForbiddenErrorJson, InternalServerErrorJson, NotFoundErrorJson,
    UnauthorizedErrorJson,
};
use crate::endpoints::json::product_category_json::*;
use crate::infrastructure::mapper::{Mapper, ProductCategoryMapper};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use business::commons::entity_mapper::EntityMapper;
use business::domain::enums::Role;
use business::domain::product_category::ProductCategoryEntityMapper;
use business::domain::user::User;
use business::gateway::product_category_gateway::ProductCategoryGateway;
use business::sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use business::use_cases::product_category_use_case::ProductCategoryUseCase;
use entity::product_category_entity;

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser | Role::Customer => user.tenant_id,
    }
}

fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

const PRODUCT_CATEGORY_SORT_FIELDS: &[&str] = &[
    "id",
    "productId",
    "product_id",
    "categoryId",
    "category_id",
    "isPrimary",
    "is_primary",
    "createdAt",
    "created_at",
];

#[utoipa::path(
    get,
    path = "/product-categories",
    tag = "ProductCategory",
    responses(
        (status = 200, description = "List of product category assignments", body = [ProductCategoryJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
) -> Json<Vec<ProductCategoryJson>> {
    let usecase = ProductCategoryUseCase::new(ProductCategoryGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_all().await;
    Json(ProductCategoryMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/product-categories/paged",
    tag = "ProductCategory",
    params(ProductCategoryPageQuery),
    responses(
        (status = 200, description = "Paged product categories", body = PagedResponse<ProductCategoryJson>),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn paged(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(params): Query<ProductCategoryPageQuery>,
) -> Json<PagedResponse<ProductCategoryJson>> {
    let norm =
        NormalizedPagination::new(&params.to_page_query(), PRODUCT_CATEGORY_SORT_FIELDS, "id");
    let mut query = product_category_entity::Entity::find();

    if current_user.role != Role::SysAdmin {
        if let Some(id) = current_user.tenant_id {
            query = query.filter(product_category_entity::Column::TenantId.eq(id));
        } else {
            return Json(PagedResponse::new(Vec::new(), 0, norm.page, norm.page_size));
        }
    } else if let Some(tenant_id) = params.tenant_id {
        query = query.filter(product_category_entity::Column::TenantId.eq(tenant_id));
    }

    if let Some(product_id) = params.product_id {
        query = query.filter(product_category_entity::Column::ProductId.eq(product_id));
    }
    if let Some(category_id) = params.category_id {
        query = query.filter(product_category_entity::Column::CategoryId.eq(category_id));
    }
    if let Some(is_primary) = params.is_primary {
        query = query.filter(product_category_entity::Column::IsPrimary.eq(is_primary));
    }

    let sort_col = match norm.sort_by.to_ascii_lowercase().as_str() {
        "productid" | "product_id" => product_category_entity::Column::ProductId,
        "categoryid" | "category_id" => product_category_entity::Column::CategoryId,
        "isprimary" | "is_primary" => product_category_entity::Column::IsPrimary,
        "createdat" | "created_at" => product_category_entity::Column::CreatedAt,
        _ => product_category_entity::Column::Id,
    };

    query = if norm.sort_dir.is_descending() {
        query
            .order_by_desc(sort_col)
            .order_by_desc(product_category_entity::Column::Id)
    } else {
        query
            .order_by_asc(sort_col)
            .order_by_asc(product_category_entity::Column::Id)
    };

    let total = query.clone().count(state.conn.as_ref()).await.unwrap_or(0);
    let rows = query
        .offset(norm.offset)
        .limit(norm.page_size)
        .all(state.conn.as_ref())
        .await
        .unwrap_or_default();

    let domains = ProductCategoryEntityMapper::from_models(rows);
    let items = ProductCategoryMapper::json_vec(domains);
    Json(PagedResponse::new(items, total, norm.page, norm.page_size))
}

#[utoipa::path(
    get,
    path = "/product-categories/{id}",
    tag = "ProductCategory",
    params(("id" = i64, Path, description = "Product Category ID")),
    responses(
        (status = 200, description = "Product category found", body = ProductCategoryJson),
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
) -> HttpResponse<Json<ProductCategoryJson>> {
    let usecase = ProductCategoryUseCase::new(ProductCategoryGateway::new(state.conn.as_ref().clone()));
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

    Ok(Json(ProductCategoryMapper::json(item)))
}

#[utoipa::path(
    get,
    path = "/product-categories/by-product/{product_id}",
    tag = "ProductCategory",
    params(("product_id" = i64, Path, description = "Product ID")),
    responses(
        (status = 200, description = "List of product category assignments for product", body = [ProductCategoryJson]),
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
) -> Json<Vec<ProductCategoryJson>> {
    let usecase = ProductCategoryUseCase::new(ProductCategoryGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_by_product_id(product_id).await;
    Json(ProductCategoryMapper::json_vec(items))
}

#[utoipa::path(
    get,
    path = "/product-categories/by-category/{category_id}",
    tag = "ProductCategory",
    params(("category_id" = i64, Path, description = "Category ID")),
    responses(
        (status = 200, description = "List of product category assignments for category", body = [ProductCategoryJson]),
        (status = 401, description = "Unauthorized", body = UnauthorizedErrorJson),
        (status = 403, description = "Forbidden", body = ForbiddenErrorJson),
        (status = 500, description = "Internal server error", body = InternalServerErrorJson),
    ),
    security(("bearer_auth" = []))
)]
pub async fn by_category(
    State(state): State<AppState>,
    Extension(_current_user): Extension<User>,
    Path(category_id): Path<i64>,
) -> Json<Vec<ProductCategoryJson>> {
    let usecase = ProductCategoryUseCase::new(ProductCategoryGateway::new(state.conn.as_ref().clone()));
    let items = usecase.find_by_category_id(category_id).await;
    Json(ProductCategoryMapper::json_vec(items))
}

#[utoipa::path(
    post,
    path = "/product-categories",
    tag = "ProductCategory",
    request_body = ProductCategoryInputJson,
    responses(
        (status = 201, description = "Product category assignment created", body = ProductCategoryJson),
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
    Json(input): Json<ProductCategoryInputJson>,
) -> HttpResponse<(StatusCode, Json<ProductCategoryJson>)> {
    let tenant_id = tenant_for_write(&user, input.tenant_id).ok_or(
        ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue),
    )?;

    if input.product_id <= 0 || input.category_id <= 0 {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let pc = ProductCategoryMapper::domain(ProductCategoryJson {
        id: 0,
        uuid: String::new(),
        tenant_id: Some(tenant_id),
        product_id: input.product_id,
        category_id: input.category_id,
        is_primary: input.is_primary,
        created_at: None,
        created_by: None,
    });

    let usecase = ProductCategoryUseCase::new(ProductCategoryGateway::new(state.conn.as_ref().clone()));
    let saved = usecase
        .create(pc)
        .await
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    Ok((StatusCode::CREATED, Json(ProductCategoryMapper::json(saved))))
}

#[utoipa::path(
    put,
    path = "/product-categories/{id}",
    tag = "ProductCategory",
    params(("id" = i64, Path, description = "Product Category ID")),
    request_body = ProductCategoryInputJson,
    responses(
        (status = 200, description = "Product category updated", body = ProductCategoryJson),
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
    Json(input): Json<ProductCategoryInputJson>,
) -> HttpResponse<Json<ProductCategoryJson>> {
    let usecase = ProductCategoryUseCase::new(ProductCategoryGateway::new(state.conn.as_ref().clone()));
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
        || input.category_id <= 0
    {
        return Err(ExceptionResponse::Forbidden(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let pc = ProductCategoryMapper::domain(ProductCategoryJson {
        id,
        uuid: existing.uuid.unwrap_or_default(),
        tenant_id: existing.tenant_id,
        product_id: input.product_id,
        category_id: input.category_id,
        is_primary: input.is_primary,
        created_at: existing.created_at,
        created_by: existing.created_by,
    });

    let saved = usecase
        .update(id, pc)
        .await
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ))?;

    Ok(Json(ProductCategoryMapper::json(saved)))
}

#[utoipa::path(
    delete,
    path = "/product-categories/{id}",
    tag = "ProductCategory",
    params(("id" = i64, Path, description = "Product Category ID")),
    responses(
        (status = 204, description = "Product category deleted"),
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
    let usecase = ProductCategoryUseCase::new(ProductCategoryGateway::new(state.conn.as_ref().clone()));
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
