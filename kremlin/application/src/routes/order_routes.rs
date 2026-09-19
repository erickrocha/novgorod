use crate::AppState;
use crate::authentication::authentication_middleware::authentication;
use crate::endpoints::order_item_endpoint as item_ep;
use crate::endpoints::order_status_history_endpoint as history_ep;
use crate::endpoints::orders_endpoint as order_ep;
use axum::routing::{get, post, put};
use axum::{Router, middleware};

pub fn order_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Orders
        .route("/orders", get(order_ep::list_all))
        .route("/orders/paged", get(order_ep::paged))
        .route("/orders", post(order_ep::add))
        .route("/orders/{id}", get(order_ep::get_by_id))
        .route("/orders/{id}", put(order_ep::update))
        // Order Items
        .route("/order-items", get(item_ep::list_all))
        .route("/order-items/paged", get(item_ep::paged))
        .route("/order-items", post(item_ep::add))
        .route("/order-items/{id}", get(item_ep::get_by_id))
        .route("/order-items/{id}", put(item_ep::update))
        // Order Status Histories
        .route("/order-status-histories", get(history_ep::list_all))
        .route("/order-status-histories/paged", get(history_ep::paged))
        .route("/order-status-histories", post(history_ep::add))
        .route("/order-status-histories/{id}", get(history_ep::get_by_id))
        .route_layer(middleware::from_fn_with_state(state, authentication))
}
