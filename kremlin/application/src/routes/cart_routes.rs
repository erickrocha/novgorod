use crate::AppState;
use crate::authentication::authentication_middleware::authentication;
use crate::endpoints::cart_endpoint as cart_ep;
use crate::endpoints::cart_item_endpoint as item_ep;
use axum::routing::{get, post, put};
use axum::{Router, middleware};

pub fn cart_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Carts
        .route("/carts", get(cart_ep::list_all))
        .route("/carts/paged", get(cart_ep::paged))
        .route("/carts", post(cart_ep::add))
        .route("/carts/{id}", get(cart_ep::get_by_id))
        .route("/carts/{id}", put(cart_ep::update))
        // Cart Items
        .route("/cart-items", get(item_ep::list_all))
        .route("/cart-items/paged", get(item_ep::paged))
        .route("/cart-items", post(item_ep::add))
        .route("/cart-items/{id}", get(item_ep::get_by_id))
        .route("/cart-items/{id}", put(item_ep::update))
        .route_layer(middleware::from_fn_with_state(state, authentication))
}
