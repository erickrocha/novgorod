use crate::{
    AppState, authentication::authentication_middleware::authentication,
    endpoints::payment_settings_endpoint as ep,
};
use axum::{Router, middleware, routing::get};

pub fn payment_settings_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/tenants/{tenantId}/payment-settings",
            get(ep::get).put(ep::put),
        )
        .route_layer(middleware::from_fn_with_state(state, authentication))
}
