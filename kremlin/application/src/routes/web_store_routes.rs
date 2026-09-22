use crate::AppState;
use crate::endpoints::web_store_endpoint::{catalog_attributes, categories, categories_paged, product_categories, products, query_products, sku_attribute_values, sku_attributes, sku_stocks, skus, skus_paged};
use axum::routing::get;
use axum::Router;

pub fn web_store_routes() -> Router<AppState> {
    Router::new()
        .route("/products", get(products))
        .route("/products/query", get(query_products))
        .route("/categories", get(categories))
        .route("/categories/paged", get(categories_paged))
        .route("/skus", get(skus))
        .route("/skus/paged", get(skus_paged))
        .route("/catalog-attributes", get(catalog_attributes))
        .route("/product-categories", get(product_categories))
        .route("/sku-attributes", get(sku_attributes))
        .route("/sku-attribute-values", get(sku_attribute_values))
        .route("/sku-stocks", get(sku_stocks))
}
