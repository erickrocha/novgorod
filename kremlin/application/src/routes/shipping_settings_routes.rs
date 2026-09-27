use crate::{
    AppState, authentication::authentication_middleware::authentication,
    endpoints::shipping_settings_endpoint as ep,
};
use axum::{Router, middleware, routing::get};
pub fn shipping_settings_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/tenants/{tenantId}/shipping-settings",
            get(ep::get).put(ep::put),
        )
        .route_layer(middleware::from_fn_with_state(state, authentication))
}
