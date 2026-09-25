use crate::AppState;
use crate::commons::pagination::PagedResponse;
use crate::endpoints::catalog_attribute_endpoint;
use crate::endpoints::category_endpoint;
use crate::endpoints::json::product_json::ProductJson;
use crate::endpoints::json::web_store_json::{
    WebStoreAttributeJson, WebStoreImageJson, WebStorePageQuery, WebStoreProductDetailJson,
    WebStoreProductJson, WebStoreSellerJson, WebStoreSellerSummaryJson, WebStoreSkuAttributeValueJson, WebStoreSkuJson,
};
use crate::endpoints::product_category_endpoint;
use crate::endpoints::sku_attribute_endpoint;
use crate::endpoints::sku_endpoint;
use crate::endpoints::sku_stock_endpoint;
use crate::infrastructure::mapper::{Mapper, ProductMapper};
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
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
        (status = 200, description = "Public List of all products", body = [WebStoreProductJson])
    ),
    security(("bearer_auth" = []))
)]
pub async fn query_products(
    state: State<AppState>,
    headers: HeaderMap,
    query: Query<WebStorePageQuery>,
) -> Result<Json<PagedResponse<WebStoreProductJson>>, StatusCode> {
    let user = build_dummy_user(&headers);
    let use_case = ProductUseCase::new(ProductGateway::new(state.conn.as_ref().clone()));

    let search_query = business::domain::product::ProductSearchQuery {
        cursor: query.cursor,
        limit: query.limit,
        q: query.q.clone(),
        active: Some(true),
        brand: None, // Can add if needed
        category: query.category.clone(),
        min_price: query.min_price,
        max_price: query.max_price,
        sort_by: query.sort_by.clone(),
    };

    let (products, next_cursor, total) = use_case.find_webstore_products(user.tenant_id, search_query)
        .await
        .map_err(|error| {
            log::error!("Failed to query webstore products: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Convert domain DTOs to API JSON DTOs
    let json_products = products.into_iter().map(|p| WebStoreProductJson {
        id: p.id,
        uuid: p.uuid,
        name: p.name,
        slug: p.slug,
        description: p.description,
        brand: p.brand,
        price_cents: p.price_cents,
        compare_at_price_cents: p.compare_at_price_cents,
        primary_image_url: p.primary_image_url.map(|key| state.storage.get_cdn_url(&key)),
        primary_image_alt: p.primary_image_alt,
        category_slugs: p.category_slugs,
        rating: p.rating,
        review_count: p.review_count,
        is_featured: p.is_featured,
        is_new: p.is_new,
        seller: p.seller.map(|seller| WebStoreSellerSummaryJson {
            id: seller.id,
            business_name: seller.business_name,
        }),
    }).collect();

    let mut response = PagedResponse::page_by_cursor(json_products, next_cursor);
    response.total = Some(total);
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/public/products/{slug}",
    tag = "WebStore",
    params(
        ("slug" = String, Path, description = "Product slug or ID")
    ),
    responses(
        (status = 200, description = "Product details with images, skus, attributes and seller", body = WebStoreProductDetailJson),
        (status = 404, description = "Product not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn product_detail(
    state: State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<WebStoreProductDetailJson>, StatusCode> {
    let use_case = ProductUseCase::new(ProductGateway::new(state.conn.as_ref().clone()));
    let detail = use_case.find_webstore_product_detail(&slug).await;

    match detail {
        Some(p) => {
            let images = p.images.into_iter().map(|img| WebStoreImageJson {
                id: img.id,
                url: state.storage.get_cdn_url(&img.object_key),
                alt_text: img.alt_text,
                sort_order: img.sort_order,
                is_primary: img.is_primary,
                width_px: img.width_px,
                height_px: img.height_px,
            }).collect();

            let skus = p.skus.into_iter().map(|s| WebStoreSkuJson {
                id: s.id,
                uuid: s.uuid,
                code: s.code,
                variant_key: s.variant_key,
                price_cents: s.price_cents,
                compare_at_price_cents: s.compare_at_price_cents,
                weight_g: s.weight_g,
                width_mm: s.width_mm,
                height_mm: s.height_mm,
                length_mm: s.length_mm,
                active: s.active,
                stock: s.stock,
                attributes: s.attributes.into_iter().map(|a| WebStoreSkuAttributeValueJson {
                    attribute_id: a.attribute_id,
                    name: a.name,
                    value: a.value,
                }).collect(),
            }).collect();

            let attributes = p.attributes.into_iter().map(|a| WebStoreAttributeJson {
                id: a.id,
                attribute_id: a.attribute_id,
                name: a.name,
                display_type: a.display_type,
                required: a.required,
                sort_order: a.sort_order,
            }).collect();

            let seller = WebStoreSellerJson {
                id: p.seller.id,
                business_name: p.seller.business_name,
                company_name: p.seller.company_name,
                email: p.seller.email,
                phone: p.seller.phone,
                web_site: p.seller.web_site,
                locality: p.seller.locality,
                administrative_area: p.seller.administrative_area,
                postal_code: p.seller.postal_code,
                country_code: p.seller.country_code,
            };

            Ok(Json(WebStoreProductDetailJson {
                id: p.id,
                uuid: p.uuid,
                name: p.name,
                slug: p.slug,
                description: p.description,
                brand: p.brand,
                active: p.active,
                ncm: p.ncm,
                cest: p.cest,
                origem_mercadoria: p.origem_mercadoria,
                seller,
                images,
                skus,
                attributes,
                category_slugs: p.category_slugs,
                rating: p.rating,
                review_count: p.review_count,
            }))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
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

#[cfg(test)]
#[path = "../../tests/unit/endpoints/web_store_endpoint.rs"]
mod tests;
