use crate::{AppState, authentication::authentication_middleware::authentication, endpoints::{orders_endpoint as ep, checkout_quote_endpoint as quote_ep, checkout_payment_endpoint as pay_ep}};
use axum::{Router, middleware, routing::{get, post}};

pub fn order_routes(state: AppState) -> Router<AppState> {
 Router::new()
 .route("/purchases", post(ep::create_purchase))
 .route("/checkout/quotes", post(quote_ep::quote))
 .route("/checkout/payment-config", get(pay_ep::config))
 .route("/purchases/{id}/payments/submit", post(pay_ep::submit))
 .route("/purchases/{id}/payments/status", get(pay_ep::status))
 .route("/purchases/paged", get(ep::purchases_paged))
 .route("/purchases/{id}", get(ep::get_purchase))
 .route("/purchases/{id}/payments", get(ep::payments))
 .route("/orders", get(ep::list_all))
 .route("/orders/paged", get(ep::paged))
 .route("/orders/{id}", get(ep::get_by_id))
 .route("/orders/{id}/status-history", get(ep::history))
 .route("/payments/{id}/transactions", get(ep::transactions))
 .route_layer(middleware::from_fn_with_state(state, authentication))
}
