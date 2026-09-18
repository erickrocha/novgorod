use crate::AppState;
use crate::authentication::authentication_middleware::authentication;
use crate::endpoints::catalog_endpoint::{
    add_category, add_product, add_sku, attribute_values, attribute_values_paged, attributes,
    attributes_paged, categories, categories_paged, product_attributes, product_attributes_paged,
    products, products_paged, skus, skus_paged, update_category, update_product, update_sku,
};
use crate::endpoints::catalog_import_endpoint::import as import_catalog;
use crate::endpoints::city_endpoint::{
    add as add_city, import_csv as import_cities, paged as paged_cities, update as update_city,
};
use crate::endpoints::city_endpoint::{
    get_by_id as get_city_by_id, get_by_province, list_all as list_cities,
};
use crate::endpoints::province_endpoint::{
    add as add_province, get_by_id as get_province_by_id, import_csv as import_provinces,
    list_all as list_provinces, paged as paged_provinces, update as update_province,
};
use axum::routing::get;
use axum::{Router, middleware};

pub fn resources_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/cities", get(list_cities))
        .route("/cities/paged", get(paged_cities))
        .route("/cities/by-province/{province_id}", get(get_by_province))
        .route("/city/by-province/{province_id}", get(get_by_province))
        .route("/city/{id}", get(get_city_by_id))
        .route("/city/paged", get(paged_cities))
        .route("/province", get(list_provinces))
        .route("/province/paged", get(paged_provinces))
        .route("/province", axum::routing::post(add_province))
        .route("/province/{id}", get(get_province_by_id))
        .route("/province/{id}", axum::routing::put(update_province))
        .route("/province/import", axum::routing::post(import_provinces))
        .route("/city", axum::routing::post(add_city))
        .route("/city/{id}", axum::routing::put(update_city))
        .route("/city/import", axum::routing::post(import_cities))
        .route("/categories", get(categories))
        .route("/categories/paged", get(categories_paged))
        .route("/catalog/import", axum::routing::post(import_catalog))
        .route("/categories", axum::routing::post(add_category))
        .route("/categories/{id}", axum::routing::put(update_category))
        .route("/catalog-attributes", get(attributes))
        .route("/catalog-attributes/paged", get(attributes_paged))
        .route("/catalog-attribute-values", get(attribute_values))
        .route("/catalog-attribute-values/paged", get(attribute_values_paged))
        .route("/products", get(products))
        .route("/products/paged", get(products_paged))
        .route("/products", axum::routing::post(add_product))
        .route("/products/{id}", axum::routing::put(update_product))
        .route("/product-attributes", get(product_attributes))
        .route("/product-attributes/paged", get(product_attributes_paged))
        .route("/skus", get(skus))
        .route("/skus/paged", get(skus_paged))
        .route("/skus", axum::routing::post(add_sku))
        .route("/skus/{id}", axum::routing::put(update_sku))
        .route_layer(middleware::from_fn_with_state(state, authentication))
}
