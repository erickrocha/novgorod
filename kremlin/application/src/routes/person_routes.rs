use crate::AppState;
use crate::authentication::authentication_middleware::authentication;
use crate::endpoints::person_address_endpoint as address_ep;
use crate::endpoints::person_endpoint as person_ep;
use axum::routing::{delete, get, post, put};
use axum::{Router, middleware};

pub fn person_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Persons
        .route("/persons", get(person_ep::list_all))
        .route("/persons/paged", get(person_ep::paged))
        .route("/persons", post(person_ep::add))
        .route("/persons/{id}", get(person_ep::get_by_id))
        .route("/persons/{id}", put(person_ep::update))
        .route("/persons/{id}", delete(person_ep::delete))
        // Person Addresses
        .route("/person-addresses", get(address_ep::list_all))
        .route("/person-addresses/paged", get(address_ep::paged))
        .route("/person-addresses/by-person/{person_id}", get(address_ep::by_person))
        .route("/person-addresses", post(address_ep::add))
        .route("/person-addresses/{id}", get(address_ep::get_by_id))
        .route("/person-addresses/{id}", put(address_ep::update))
        .route("/person-addresses/{id}", delete(address_ep::delete))
        .route_layer(middleware::from_fn_with_state(state, authentication))
}
