use crate::AppState;
use crate::commons::exception_response::{ExceptionResponse, HttpResponse};
use crate::commons::i18n::{ErrorKey, Locale};
use crate::endpoints::json::product_image_json::{
    ProductImageJson, ProductImagePresignBatchRequest, ProductImagePresignItemResponse,
};
use crate::infrastructure::mapper::{Mapper, ProductImageMapper};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::product_image_gateway::ProductImageGateway;
use business::use_cases::product_image_use_case::{
    CreateImageUploadRequest, ProductImageUseCase,
};
use business::sea_orm::EntityTrait;
use entity::product_entity;


fn can_read_tenant(user: &User, tenant_id: Option<i64>) -> bool {
    user.role == Role::SysAdmin || (user.tenant_id.is_some() && user.tenant_id == tenant_id)
}

fn tenant_for_write(user: &User, requested: Option<i64>) -> Option<i64> {
    match user.role {
        Role::SysAdmin => requested,
        Role::TenantOwner | Role::TenantUser => user.tenant_id,
    }
}

#[utoipa::path(
    post,
    path = "/products/{id}/images/presign",
    tag = "Catalog",
    params(("id" = i64, Path, description = "Product ID")),
    request_body = ProductImagePresignBatchRequest,
    responses(
        (status = 201, description = "Presigned upload URLs generated", body = Vec<ProductImagePresignItemResponse>),
        (status = 400, description = "Invalid parameter"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Product not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn presign(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(body): Json<ProductImagePresignBatchRequest>,
) -> HttpResponse<(StatusCode, Json<Vec<ProductImagePresignItemResponse>>)> {
    let product = product_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?;

    let tenant_id = tenant_for_write(&user, product.tenant_id)
        .ok_or(ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue))?;

    if body.images.is_empty() {
        return Err(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::InvalidParameterValue,
        ));
    }

    let use_case = ProductImageUseCase::new(
        ProductImageGateway::new(state.conn.as_ref().clone()),
        (*state.storage).clone(),
    );

    let requests: Vec<CreateImageUploadRequest> = body
        .images
        .into_iter()
        .map(|img| CreateImageUploadRequest {
            original_filename: img.original_filename,
            mime_type: img.mime_type,
            size_bytes: img.size_bytes,
            sku_id: img.sku_id,
            alt_text: img.alt_text,
            is_primary: img.is_primary,
            sort_order: img.sort_order,
        })
        .collect();

    let results = use_case
        .request_batch_presigned_upload(id, Some(tenant_id), requests)
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    let response_items = results
        .into_iter()
        .map(|r| ProductImagePresignItemResponse {
            upload_url: r.upload_url,
            object_key: r.image.object_key.clone(),
            cdn_url: r.image.cdn_url.clone(),
            image: ProductImageMapper::json(r.image),
        })
        .collect();

    Ok((StatusCode::CREATED, Json(response_items)))
}

#[utoipa::path(
    get,
    path = "/products/{id}/images",
    tag = "Catalog",
    params(("id" = i64, Path, description = "Product ID")),
    responses(
        (status = 200, description = "List of product images", body = Vec<ProductImageJson>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Product not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<Vec<ProductImageJson>>> {
    let product = product_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?;

    if !can_read_tenant(&user, product.tenant_id) {
        return Err(ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue));
    }

    let use_case = ProductImageUseCase::new(
        ProductImageGateway::new(state.conn.as_ref().clone()),
        (*state.storage).clone(),
    );

    let images = use_case
        .list_by_product(id)
        .await
        .map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::InvalidParameterValue))?;

    Ok(Json(ProductImageMapper::json_vec(images)))
}

#[utoipa::path(
    delete,
    path = "/products/{id}/images/{image_id}",
    tag = "Catalog",
    params(
        ("id" = i64, Path, description = "Product ID"),
        ("image_id" = i64, Path, description = "Image ID")
    ),
    responses(
        (status = 204, description = "Image deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Image or product not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path((id, image_id)): Path<(i64, i64)>,
) -> HttpResponse<StatusCode> {
    let product = product_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?;

    let _ = tenant_for_write(&user, product.tenant_id)
        .ok_or(ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue))?;

    let use_case = ProductImageUseCase::new(
        ProductImageGateway::new(state.conn.as_ref().clone()),
        (*state.storage).clone(),
    );

    use_case
        .delete_image(id, image_id)
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/products/{id}/images/{image_id}/primary",
    tag = "Catalog",
    params(
        ("id" = i64, Path, description = "Product ID"),
        ("image_id" = i64, Path, description = "Image ID")
    ),
    responses(
        (status = 200, description = "Image set as primary"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Image or product not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn set_primary(
    State(state): State<AppState>,
    Extension(locale): Extension<Locale>,
    Extension(user): Extension<User>,
    Path((id, image_id)): Path<(i64, i64)>,
) -> HttpResponse<StatusCode> {
    let product = product_entity::Entity::find_by_id(id)
        .one(state.conn.as_ref())
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?
        .ok_or(ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?;

    let _ = tenant_for_write(&user, product.tenant_id)
        .ok_or(ExceptionResponse::Forbidden(locale, ErrorKey::InvalidParameterValue))?;

    let use_case = ProductImageUseCase::new(
        ProductImageGateway::new(state.conn.as_ref().clone()),
        (*state.storage).clone(),
    );

    use_case
        .set_primary(id, image_id)
        .await
        .map_err(|_| ExceptionResponse::NotFound(locale, ErrorKey::InvalidParameterValue))?;

    Ok(StatusCode::OK)
}
