use crate::AppState;
use crate::commons::pagination::{CursorPageQuery, PagedResponse};
use crate::endpoints::catalog_attribute_endpoint;
use crate::endpoints::category_endpoint;
use crate::endpoints::json::product_json::ProductJson;
use crate::endpoints::product_category_endpoint;
use crate::endpoints::sku_attribute_endpoint;
use crate::endpoints::sku_endpoint;
use crate::endpoints::sku_stock_endpoint;
use crate::infrastructure::mapper::{Mapper, ProductMapper};
use axum::{
    Json,
    extract::{Extension, Query, State},
    http::HeaderMap,
};
use business::domain::enums::Role;
use business::domain::user::User;
use business::gateway::product_gateway::ProductGateway;
use business::use_cases::product_use_case::ProductUseCase;

fn build_dummy_user(headers: &HeaderMap) -> User {
    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    User {
        id: None,
        uuid: None,
        email: "".to_string(),
        name: None,
        password: "".to_string(),
        enabled: true,
        first_login: false,
        tenant_id,
        role: Role::TenantUser,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    }
}

#[utoipa::path(
    get,
    path = "/api/public/products",
    tag = "WebStore",
    responses(
        (status = 200, description = "Public List of all products", body = [ProductJson])
    ),
    security(("bearer_auth" = []))
)]
pub async fn products(state: State<AppState>) -> Json<Vec<ProductJson>> {
    let use_case = ProductUseCase::new(ProductGateway::new(state.conn.as_ref().clone()));
    let products = use_case.find_all().await;
    Json(ProductMapper::json_vec(products))
}

#[utoipa::path(
    get,
    path = "/api/public/products/query",
    tag = "WebStore",
    responses(
        (status = 200, description = "Public List of all products", body = [ProductJson])
    ),
    security(("bearer_auth" = []))
)]
pub async fn query_products(state: State<AppState>, query: Query<CursorPageQuery>) -> Json<PagedResponse<ProductJson>> {
    let use_case = ProductUseCase::new(ProductGateway::new(state.conn.as_ref().clone()));

    let search_query = business::domain::product::ProductSearchQuery {
        cursor: query.cursor,
        limit: query.limit,
        q: query.q.clone(),
        active: query.active,
        brand: query.brand.clone(),
    };

    let (products, next_cursor) = use_case.find_paged_by_cursor(search_query).await;

    Json(PagedResponse::page_by_cursor(ProductMapper::json_vec(products), next_cursor))
}

// ==========================================
// Categories
// ==========================================
pub async fn categories(
    state: State<AppState>,
    headers: HeaderMap,
) -> Json<Vec<crate::endpoints::json::catalog_json::CategoryJson>> {
    let user = build_dummy_user(&headers);
    category_endpoint::categories(state, Extension(user)).await
}

pub async fn categories_paged(
    state: State<AppState>,
    headers: HeaderMap,
    query: Query<category_endpoint::CategoryPageQuery>,
) -> Json<crate::commons::pagination::PagedResponse<crate::endpoints::json::catalog_json::CategoryJson>> {
    let user = build_dummy_user(&headers);
    category_endpoint::categories_paged(state, Extension(user), query).await
}

// ==========================================
// SKUs
// ==========================================
pub async fn skus(
    state: State<AppState>,
    headers: HeaderMap,
) -> Json<Vec<crate::endpoints::json::catalog_json::SkuJson>> {
    let user = build_dummy_user(&headers);
    sku_endpoint::skus(state, Extension(user)).await
}

pub async fn skus_paged(
    state: State<AppState>,
    headers: HeaderMap,
    query: Query<sku_endpoint::SkuPageQuery>,
) -> Json<crate::commons::pagination::PagedResponse<crate::endpoints::json::catalog_json::SkuJson>> {
    let user = build_dummy_user(&headers);
    sku_endpoint::skus_paged(state, Extension(user), query).await
}

// ==========================================
// Catalog Attributes
// ==========================================
pub async fn catalog_attributes(
    state: State<AppState>,
    headers: HeaderMap,
) -> Json<Vec<crate::endpoints::json::catalog_json::CatalogAttributeJson>> {
    let user = build_dummy_user(&headers);
    catalog_attribute_endpoint::attributes(state, Extension(user)).await
}

// ==========================================
// Product Categories
// ==========================================
pub async fn product_categories(
    state: State<AppState>,
    headers: HeaderMap,
) -> Json<Vec<crate::endpoints::json::product_category_json::ProductCategoryJson>> {
    let user = build_dummy_user(&headers);
    product_category_endpoint::list_all(state, Extension(user)).await
}

// ==========================================
// SKU Attributes
// ==========================================
pub async fn sku_attributes(
    state: State<AppState>,
    headers: HeaderMap,
) -> Json<Vec<crate::endpoints::json::sku_attribute_json::SkuAttributeJson>> {
    let user = build_dummy_user(&headers);
    sku_attribute_endpoint::list_all(state, Extension(user)).await
}

// ==========================================
// SKU Attribute Values
// ==========================================
pub async fn sku_attribute_values(
    state: State<AppState>,
    headers: HeaderMap,
) -> Json<Vec<crate::endpoints::json::sku_attribute_json::SkuAttributeJson>> {
    let user = build_dummy_user(&headers);
    sku_attribute_endpoint::list_all(state, Extension(user)).await
}

// ==========================================
// SKU Stocks
// ==========================================
pub async fn sku_stocks(
    state: State<AppState>,
    headers: HeaderMap,
) -> Json<Vec<crate::endpoints::json::sku_stock_json::SkuStockJson>> {
    let user = build_dummy_user(&headers);
    sku_stock_endpoint::list_all(state, Extension(user)).await
}
