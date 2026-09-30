use crate::AppState;
use crate::authentication::authentication_middleware::authentication;
use crate::endpoints::shipping_rate_endpoint as ship_ep;
use crate::endpoints::tax_rule_endpoint as tax_ep;
use crate::endpoints::warehouse_endpoint as wh_ep;
use axum::routing::{delete, get, post, put};
use axum::{Router, middleware};

pub fn shipping_tax_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Warehouses
        .route("/warehouses", get(wh_ep::list_all))
        .route("/warehouses/paged", get(wh_ep::paged))
        .route("/warehouses", post(wh_ep::add))
        .route("/warehouses/{id}", get(wh_ep::get_by_id))
        .route("/warehouses/{id}", put(wh_ep::update))
        .route("/warehouses/{id}", delete(wh_ep::delete))
        // Shipping Rates
        .route("/shipping-rates", get(ship_ep::list_all))
        .route("/shipping-rates/paged", get(ship_ep::paged))
        .route("/shipping-rates", post(ship_ep::add))
        .route("/shipping-rates/{id}", get(ship_ep::get_by_id))
        .route("/shipping-rates/{id}", put(ship_ep::update))
        .route("/shipping-rates/{id}", delete(ship_ep::delete))
        // Tax Rules
        .route("/tax-rules", get(tax_ep::list_all))
        .route("/tax-rules/paged", get(tax_ep::paged))
        .route("/tax-rules", post(tax_ep::add))
        .route("/tax-rules/{id}", get(tax_ep::get_by_id))
        .route("/tax-rules/{id}", put(tax_ep::update))
        .route_layer(middleware::from_fn_with_state(state, authentication))
}
