use crate::AppState;
use crate::authentication::authentication_middleware::authentication;
use crate::endpoints::shipping_rate_endpoint as ship_ep;
use crate::endpoints::tax_rule_endpoint as tax_ep;
use axum::routing::{get, post, put};
use axum::{Router, middleware};

pub fn shipping_tax_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Shipping Rates
        .route("/shipping-rates", get(ship_ep::list_all))
        .route("/shipping-rates/paged", get(ship_ep::paged))
        .route("/shipping-rates", post(ship_ep::add))
        .route("/shipping-rates/{id}", get(ship_ep::get_by_id))
        .route("/shipping-rates/{id}", put(ship_ep::update))
        // Tax Rules
        .route("/tax-rules", get(tax_ep::list_all))
        .route("/tax-rules/paged", get(tax_ep::paged))
        .route("/tax-rules", post(tax_ep::add))
        .route("/tax-rules/{id}", get(tax_ep::get_by_id))
        .route("/tax-rules/{id}", put(tax_ep::update))
        .route_layer(middleware::from_fn_with_state(state, authentication))
}
