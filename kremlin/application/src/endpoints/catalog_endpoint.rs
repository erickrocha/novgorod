use crate::AppState;
use crate::endpoints::json::catalog_json::*;
use axum::{Json, extract::State};
use business::gateway::{
    catalog_attribute_gateway::CatalogAttributeGateway, category_gateway::CategoryGateway,
    product_attribute_gateway::ProductAttributeGateway,
    product_attribute_value_gateway::ProductAttributeValueGateway, product_gateway::ProductGateway,
    sku_gateway::SkuGateway,
};
use business::use_cases::{
    catalog_attribute_use_case::CatalogAttributeUseCase, category_use_case::CategoryUseCase,
    product_attribute_use_case::ProductAttributeUseCase,
    product_attribute_value_use_case::ProductAttributeValueUseCase,
    product_use_case::ProductUseCase, sku_use_case::SkuUseCase,
};
#[utoipa::path(get, path = "/categories", tag = "Catalog", responses((status = 200, body = [CategoryJson])))]
pub async fn categories(State(state): State<AppState>) -> Json<Vec<CategoryJson>> {
    let r = CategoryUseCase::new(CategoryGateway::new(state.conn.as_ref().clone()))
        .find_all()
        .await
        .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| CategoryJson {
                id: x.id,
                uuid: x.uuid.unwrap_or_default(),
                tenant_id: x.tenant_id,
                name: x.name,
                slug: x.slug,
                parent_id: x.parent_id,
                active: x.active,
            })
            .collect(),
    )
}
#[utoipa::path(get, path = "/catalog-attributes", tag = "Catalog", responses((status = 200, body = [CatalogAttributeJson])))]
pub async fn attributes(State(state): State<AppState>) -> Json<Vec<CatalogAttributeJson>> {
    let r = CatalogAttributeUseCase::new(CatalogAttributeGateway::new(state.conn.as_ref().clone()))
        .find_all()
        .await
        .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| CatalogAttributeJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                name: x.name,
                display_type: x.display_type,
            })
            .collect(),
    )
}
#[utoipa::path(get, path = "/catalog-attribute-values", tag = "Catalog", responses((status = 200, body = [CatalogAttributeValueJson])))]
pub async fn attribute_values(
    State(state): State<AppState>,
) -> Json<Vec<CatalogAttributeValueJson>> {
    let r = ProductAttributeValueUseCase::new(ProductAttributeValueGateway::new(
        state.conn.as_ref().clone(),
    ))
    .find_all()
    .await
    .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| CatalogAttributeValueJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                attribute_id: x.attribute_id,
                value: x.value,
            })
            .collect(),
    )
}
#[utoipa::path(get, path = "/products", tag = "Catalog", responses((status = 200, body = [ProductJson])))]
pub async fn products(State(state): State<AppState>) -> Json<Vec<ProductJson>> {
    let r = ProductUseCase::new(ProductGateway::new(state.conn.as_ref().clone()))
        .find_all()
        .await
        .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| ProductJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                name: x.name,
                slug: x.slug,
                description: x.description,
                brand: x.brand,
                active: x.active,
                ncm: x.ncm,
                cest: x.cest,
                origem_mercadoria: x.origem_mercadoria,
            })
            .collect(),
    )
}
#[utoipa::path(get, path = "/product-attributes", tag = "Catalog", responses((status = 200, body = [ProductAttributeJson])))]
pub async fn product_attributes(State(state): State<AppState>) -> Json<Vec<ProductAttributeJson>> {
    let r = ProductAttributeUseCase::new(ProductAttributeGateway::new(state.conn.as_ref().clone()))
        .find_all()
        .await
        .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| ProductAttributeJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                product_id: x.product_id,
                attribute_id: x.attribute_id,
                required: x.required,
                sort_order: x.sort_order,
            })
            .collect(),
    )
}
#[utoipa::path(get, path = "/skus", tag = "Catalog", responses((status = 200, body = [SkuJson])))]
pub async fn skus(State(state): State<AppState>) -> Json<Vec<SkuJson>> {
    let r = SkuUseCase::new(SkuGateway::new(state.conn.as_ref().clone()))
        .find_all()
        .await
        .unwrap_or_default();
    Json(
        r.into_iter()
            .map(|x| SkuJson {
                id: x.id,
                uuid: x.uuid.to_string(),
                tenant_id: x.tenant_id,
                product_id: x.product_id,
                code: x.code,
                variant_key: x.variant_key,
                price_cents: x.price_cents,
                compare_at_price_cents: x.compare_at_price_cents,
                weight_g: x.weight_g,
                width_mm: x.width_mm,
                height_mm: x.height_mm,
                length_mm: x.length_mm,
                active: x.active,
            })
            .collect(),
    )
}
