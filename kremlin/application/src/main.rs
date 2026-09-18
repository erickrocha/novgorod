mod authentication;
mod commons;
mod endpoints;
mod infrastructure;
mod routes;

use crate::endpoints::welcome_endpoint::welcome;
use crate::routes::authentication_routes::auth_routes;
use crate::routes::resource_routes::resources_routes;
use crate::routes::tenant_routes::tenant_routes;
use crate::routes::user_routes::user_routes;
use axum::Router;
use axum::http::{Method, header};
use axum::routing::get;
use business::sea_orm::DatabaseConnection;
use migration::{Migrator, MigratorTrait};
use std::env;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        endpoints::tenant_endpoint::add,
        endpoints::tenant_endpoint::get_by_id,
        endpoints::tenant_endpoint::get_by_uuid,
        endpoints::tenant_endpoint::list_all,
        endpoints::tenant_endpoint::update,
        endpoints::user_endpoint::get_by_id,
        endpoints::user_endpoint::add,
        endpoints::user_endpoint::list_all,
        endpoints::user_endpoint::update,
        endpoints::user_endpoint::change_password,
        endpoints::province_endpoint::list_all,
        endpoints::province_endpoint::get_by_id,
        endpoints::province_endpoint::add,
        endpoints::province_endpoint::update,
        endpoints::province_endpoint::import_csv,
        endpoints::city_endpoint::list_all,
        endpoints::city_endpoint::get_by_province,
        endpoints::city_endpoint::get_by_id
        ,endpoints::city_endpoint::add
        ,endpoints::city_endpoint::update
        ,endpoints::city_endpoint::import_csv
        ,endpoints::catalog_endpoint::categories
        ,endpoints::catalog_endpoint::attributes
        ,endpoints::catalog_endpoint::attribute_values
        ,endpoints::catalog_endpoint::products
        ,endpoints::catalog_endpoint::product_attributes
        ,endpoints::catalog_endpoint::skus
        ,endpoints::catalog_endpoint::add_category
        ,endpoints::catalog_endpoint::update_category
        ,endpoints::catalog_endpoint::add_product
        ,endpoints::catalog_endpoint::update_product
        ,endpoints::catalog_endpoint::add_sku
        ,endpoints::catalog_endpoint::update_sku
    ),
    components(
        schemas(
            endpoints::json::user_json::UserJson,
            endpoints::json::login_request::LoginRequest,
            endpoints::json::change_password_request::ChangePasswordRequest,
            endpoints::json::refresh_token_request::RefreshTokenRequest,
            endpoints::json::access_token_json::AccessTokenJson,
            endpoints::json::tenant_json::TenantJson,
            endpoints::json::province_json::ProvinceJson,
            endpoints::json::city_json::CityJson,
            endpoints::json::catalog_json::CategoryJson,
            endpoints::json::catalog_json::CatalogAttributeJson,
            endpoints::json::catalog_json::CatalogAttributeValueJson,
            endpoints::json::catalog_json::CatalogAttributeValueJson,
            endpoints::json::catalog_json::ProductJson,
            endpoints::json::catalog_json::ProductAttributeJson,
            endpoints::json::catalog_json::SkuJson,
            endpoints::json::catalog_json::CategoryInputJson,
            endpoints::json::catalog_json::ProductInputJson,
            endpoints::json::catalog_json::SkuInputJson,
        ),
    ),
    tags(
        (name = "Hermes", description = "REST API for Hermes")
    )
)]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub conn: Arc<DatabaseConnection>,
}

// ==================== Route Builders ====================
/// Build public welcome route
fn welcome_route() -> Router<AppState> {
    Router::new().route("/", get(welcome))
}

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let log_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(log_filter).init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let connection = business::commons::db_pool::connect(&db_url, "workout-application")
        .await
        .expect("Failed to connect to database");
    Migrator::up(&connection, None).await?;

    business::use_cases::user_use_case::UserUseCase::seed_sysadmin(&connection).await;

    let state = AppState {
        conn: Arc::new(connection),
    };

    log::info!("Starting server...");

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::HEAD,
        ])
        .allow_origin(Any)
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::ORIGIN,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            header::ACCESS_CONTROL_ALLOW_METHODS,
            header::ACCESS_CONTROL_ALLOW_HEADERS,
        ]);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Public routes (no authentication)
        .merge(welcome_route())
        .merge(auth_routes(state.clone()))
        .merge(resources_routes(state.clone()))
        .nest("/tenant", tenant_routes(state.clone()))
        .nest("/user", user_routes(state.clone()))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&server_url).await?;
    log::info!("Server started on address {}", server_url);
    axum::serve(listener, app).await?;

    Ok(())
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
