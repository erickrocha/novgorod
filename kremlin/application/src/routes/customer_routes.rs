use crate::AppState;
use crate::authentication::authentication_middleware::authentication;
use crate::endpoints::customer_address_endpoint as address_ep;
use crate::endpoints::customer_endpoint as cust_ep;
use axum::routing::{get, post, put};
use axum::{Router, middleware};

pub fn customer_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Customers
        .route("/customers", get(cust_ep::list_all))
        .route("/customers/paged", get(cust_ep::paged))
        .route("/customers", post(cust_ep::add))
        .route("/customers/{id}", get(cust_ep::get_by_id))
        .route("/customers/{id}", put(cust_ep::update))
        // Customer Addresses
        .route("/customer-addresses", get(address_ep::list_all))
        .route("/customer-addresses/paged", get(address_ep::paged))
        .route("/customer-addresses", post(address_ep::add))
        .route("/customer-addresses/{id}", get(address_ep::get_by_id))
        .route("/customer-addresses/{id}", put(address_ep::update))
        .route_layer(middleware::from_fn_with_state(state, authentication))
}
