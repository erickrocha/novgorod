use crate::AppState;
use crate::authentication::authentication_middleware::authentication;
use crate::endpoints::catalog_attribute_endpoint::{
    attribute_values, attribute_values_paged, attributes, attributes_paged, product_attributes,
    product_attributes_paged,
};
use crate::endpoints::category_endpoint::{
    add_category, categories, categories_paged, update_category,
};
use crate::endpoints::product_category_endpoint as prod_cat_ep;
use crate::endpoints::product_endpoint::{
    add_product, products, products_paged, update_product,
};
use crate::endpoints::sku_attribute_endpoint as sku_attr_ep;
use crate::endpoints::sku_endpoint::{add_sku, skus, skus_paged, update_sku};
use crate::endpoints::sku_stock_endpoint as sku_stock_ep;
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
use axum::routing::{delete, get, post, put};
use axum::{Router, middleware};

pub fn resources_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Cities & Provinces
        .route("/cities", get(list_cities))
        .route("/cities/paged", get(paged_cities))
        .route("/cities/by-province/{province_id}", get(get_by_province))
        .route("/city/by-province/{province_id}", get(get_by_province))
        .route("/city/{id}", get(get_city_by_id))
        .route("/city/paged", get(paged_cities))
        .route("/province", get(list_provinces))
        .route("/province/paged", get(paged_provinces))
        .route("/province", post(add_province))
        .route("/province/{id}", get(get_province_by_id))
        .route("/province/{id}", put(update_province))
        .route("/province/import", post(import_provinces))
        .route("/city", post(add_city))
        .route("/city/{id}", put(update_city))
        .route("/city/import", post(import_cities))
        // Categories & Catalog Import
        .route("/categories", get(categories))
        .route("/categories/paged", get(categories_paged))
        .route("/catalog/import", post(import_catalog))
        .route("/categories", post(add_category))
        .route("/categories/{id}", put(update_category))
        // Catalog Attributes
        .route("/catalog-attributes", get(attributes))
        .route("/catalog-attributes/paged", get(attributes_paged))
        .route("/catalog-attribute-values", get(attribute_values))
        .route("/catalog-attribute-values/paged", get(attribute_values_paged))
        // Products
        .route("/products", get(products))
        .route("/products/paged", get(products_paged))
        .route("/products", post(add_product))
        .route("/products/{id}", put(update_product))
        .route("/product-attributes", get(product_attributes))
        .route("/product-attributes/paged", get(product_attributes_paged))
        // Product Categories
        .route("/product-categories", get(prod_cat_ep::list_all))
        .route("/product-categories/paged", get(prod_cat_ep::paged))
        .route("/product-categories/{id}", get(prod_cat_ep::get_by_id))
        .route("/product-categories/by-product/{product_id}", get(prod_cat_ep::by_product))
        .route("/product-categories/by-category/{category_id}", get(prod_cat_ep::by_category))
        .route("/product-categories", post(prod_cat_ep::add))
        .route("/product-categories/{id}", put(prod_cat_ep::update))
        .route("/product-categories/{id}", delete(prod_cat_ep::delete))
        // SKUs
        .route("/skus", get(skus))
        .route("/skus/paged", get(skus_paged))
        .route("/skus", post(add_sku))
        .route("/skus/{id}", put(update_sku))
        // SKU Attributes (and values alias)
        .route("/sku-attributes", get(sku_attr_ep::list_all))
        .route("/sku-attributes/paged", get(sku_attr_ep::paged))
        .route("/sku-attributes/{id}", get(sku_attr_ep::get_by_id))
        .route("/sku-attributes/by-sku/{sku_id}", get(sku_attr_ep::by_sku))
        .route("/sku-attributes/by-product/{product_id}", get(sku_attr_ep::by_product))
        .route("/sku-attributes", post(sku_attr_ep::add))
        .route("/sku-attributes/{id}", put(sku_attr_ep::update))
        .route("/sku-attributes/{id}", delete(sku_attr_ep::delete))
        .route("/sku-attribute-values", get(sku_attr_ep::list_all))
        .route("/sku-attribute-values/paged", get(sku_attr_ep::paged))
        .route("/sku-attribute-values/{id}", get(sku_attr_ep::get_by_id))
        .route("/sku-attribute-values/by-sku/{sku_id}", get(sku_attr_ep::by_sku))
        .route("/sku-attribute-values", post(sku_attr_ep::add))
        .route("/sku-attribute-values/{id}", put(sku_attr_ep::update))
        .route("/sku-attribute-values/{id}", delete(sku_attr_ep::delete))
        // SKU Stocks (Inventory)
        .route("/sku-stocks", get(sku_stock_ep::list_all))
        .route("/sku-stocks/paged", get(sku_stock_ep::paged))
        .route("/sku-stocks/{id}", get(sku_stock_ep::get_by_id))
        .route("/sku-stocks/by-sku/{sku_id}", get(sku_stock_ep::by_sku))
        .route("/sku-stocks", post(sku_stock_ep::add))
        .route("/sku-stocks/{id}", put(sku_stock_ep::update))
        .route("/sku-stocks/{id}", delete(sku_stock_ep::delete))
        // Product Images
        .route("/products/{id}/images/presign", post(crate::endpoints::product_image_endpoint::presign))
        .route("/products/{id}/images", get(crate::endpoints::product_image_endpoint::list))
        .route("/products/{id}/images/{image_id}", delete(crate::endpoints::product_image_endpoint::delete))
        .route("/products/{id}/images/{image_id}/primary", put(crate::endpoints::product_image_endpoint::set_primary))
        // Profile & Avatar Resources
        .route(
            "/resource/profile",
            get(crate::endpoints::resource_endpoint::get_profile)
                .put(crate::endpoints::resource_endpoint::update_profile),
        )
        .route("/resource", get(crate::endpoints::resource_endpoint::get_profile))
        .route("/resource/me", get(crate::endpoints::resource_endpoint::get_profile))
        .route(
            "/profile",
            get(crate::endpoints::resource_endpoint::get_profile)
                .put(crate::endpoints::resource_endpoint::update_profile),
        )
        .route(
            "/resource/avatar/presign",
            post(crate::endpoints::resource_endpoint::presign_avatar),
        )
        .route_layer(middleware::from_fn_with_state(state, authentication))
}
