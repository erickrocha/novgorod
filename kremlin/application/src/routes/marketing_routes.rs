use crate::AppState;
use crate::authentication::authentication_middleware::authentication;
use crate::endpoints::campaign_endpoint as camp_ep;
use crate::endpoints::campaign_target_endpoint as target_ep;
use crate::endpoints::coupon_endpoint as coup_ep;
use crate::endpoints::coupon_redemption_endpoint as redemp_ep;
use axum::routing::{get, post, put};
use axum::{Router, middleware};

pub fn marketing_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Campaigns
        .route("/campaigns", get(camp_ep::list_all))
        .route("/campaigns/paged", get(camp_ep::paged))
        .route("/campaigns", post(camp_ep::add))
        .route("/campaigns/{id}", get(camp_ep::get_by_id))
        .route("/campaigns/{id}", put(camp_ep::update))
        // Campaign Targets
        .route("/campaign-targets", get(target_ep::list_all))
        .route("/campaign-targets/paged", get(target_ep::paged))
        .route("/campaign-targets", post(target_ep::add))
        .route("/campaign-targets/{id}", get(target_ep::get_by_id))
        .route("/campaign-targets/{id}", put(target_ep::update))
        // Coupons
        .route("/coupons", get(coup_ep::list_all))
        .route("/coupons/paged", get(coup_ep::paged))
        .route("/coupons", post(coup_ep::add))
        .route("/coupons/{id}", get(coup_ep::get_by_id))
        .route("/coupons/{id}", put(coup_ep::update))
        // Coupon Redemptions
        .route("/coupon-redemptions", get(redemp_ep::list_all))
        .route("/coupon-redemptions/paged", get(redemp_ep::paged))
        .route("/coupon-redemptions", post(redemp_ep::add))
        .route("/coupon-redemptions/{id}", get(redemp_ep::get_by_id))
        .route_layer(middleware::from_fn_with_state(state, authentication))
}
