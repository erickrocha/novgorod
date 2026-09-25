mod authentication;
mod commons;
mod endpoints;
mod infrastructure;
mod routes;

use crate::endpoints::welcome_endpoint::welcome;
use crate::routes::authentication_routes::auth_routes;
use crate::routes::cart_routes::cart_routes;
use crate::routes::customer_routes::customer_routes;
use crate::routes::marketing_routes::marketing_routes;
use crate::routes::order_routes::order_routes;
use crate::routes::person_routes::person_routes;
use crate::routes::resource_routes::resources_routes;
use crate::routes::shipping_tax_routes::shipping_tax_routes;
use crate::routes::tenant_routes::tenant_routes;
use crate::routes::user_routes::user_routes;
use crate::routes::web_store_routes::web_store_routes;
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
        endpoints::tenant_endpoint::paged,
        endpoints::user_endpoint::get_by_id,
        endpoints::user_endpoint::add,
        endpoints::user_endpoint::list_all,
        endpoints::user_endpoint::paged,
        endpoints::user_endpoint::update,
        endpoints::user_endpoint::change_password,
        endpoints::province_endpoint::list_all,
        endpoints::province_endpoint::paged,
        endpoints::province_endpoint::get_by_id,
        endpoints::province_endpoint::add,
        endpoints::province_endpoint::update,
        endpoints::province_endpoint::import_csv,
        endpoints::city_endpoint::list_all,
        endpoints::city_endpoint::paged,
        endpoints::city_endpoint::get_by_province,
        endpoints::city_endpoint::get_by_id,
        endpoints::city_endpoint::add,
        endpoints::city_endpoint::update,
        endpoints::city_endpoint::import_csv,
        endpoints::category_endpoint::categories,
        endpoints::category_endpoint::categories_paged,
        endpoints::category_endpoint::add_category,
        endpoints::category_endpoint::update_category,
        endpoints::catalog_attribute_endpoint::attributes,
        endpoints::catalog_attribute_endpoint::attributes_paged,
        endpoints::catalog_attribute_endpoint::attribute_values,
        endpoints::catalog_attribute_endpoint::attribute_values_paged,
        endpoints::catalog_attribute_endpoint::product_attributes,
        endpoints::catalog_attribute_endpoint::product_attributes_paged,
        endpoints::product_endpoint::products,
        endpoints::product_endpoint::products_paged,
        endpoints::product_endpoint::add_product,
        endpoints::product_endpoint::update_product,
        endpoints::sku_endpoint::skus,
        endpoints::sku_endpoint::skus_paged,
        endpoints::sku_endpoint::add_sku,
        endpoints::sku_endpoint::update_sku,
        endpoints::product_image_endpoint::presign,
        endpoints::product_image_endpoint::list,
        endpoints::product_image_endpoint::delete,
        endpoints::product_image_endpoint::set_primary,
        endpoints::shipping_rate_endpoint::list_all,
        endpoints::shipping_rate_endpoint::paged,
        endpoints::shipping_rate_endpoint::get_by_id,
        endpoints::shipping_rate_endpoint::add,
        endpoints::shipping_rate_endpoint::update,
        endpoints::customer_endpoint::list_all,
        endpoints::customer_endpoint::paged,
        endpoints::customer_endpoint::get_by_id,
        endpoints::customer_endpoint::add,
        endpoints::customer_endpoint::update,
        endpoints::customer_address_endpoint::list_all,
        endpoints::customer_address_endpoint::paged,
        endpoints::customer_address_endpoint::get_by_id,
        endpoints::customer_address_endpoint::add,
        endpoints::customer_address_endpoint::update,
        endpoints::person_endpoint::list_all,
        endpoints::person_endpoint::paged,
        endpoints::person_endpoint::get_by_id,
        endpoints::person_endpoint::add,
        endpoints::person_endpoint::update,
        endpoints::person_endpoint::delete,
        endpoints::person_address_endpoint::list_all,
        endpoints::person_address_endpoint::paged,
        endpoints::person_address_endpoint::get_by_id,
        endpoints::person_address_endpoint::by_person,
        endpoints::person_address_endpoint::add,
        endpoints::person_address_endpoint::update,
        endpoints::person_address_endpoint::delete,
        endpoints::resource_endpoint::get_profile,
        endpoints::resource_endpoint::update_profile,
        endpoints::resource_endpoint::presign_avatar,
        endpoints::tax_rule_endpoint::list_all,
        endpoints::tax_rule_endpoint::paged,
        endpoints::tax_rule_endpoint::get_by_id,
        endpoints::tax_rule_endpoint::add,
        endpoints::tax_rule_endpoint::update,
        endpoints::campaign_endpoint::list_all,
        endpoints::campaign_endpoint::paged,
        endpoints::campaign_endpoint::get_by_id,
        endpoints::campaign_endpoint::add,
        endpoints::campaign_endpoint::update,
        endpoints::campaign_target_endpoint::list_all,
        endpoints::campaign_target_endpoint::paged,
        endpoints::campaign_target_endpoint::get_by_id,
        endpoints::campaign_target_endpoint::add,
        endpoints::campaign_target_endpoint::update,
        endpoints::coupon_endpoint::list_all,
        endpoints::coupon_endpoint::paged,
        endpoints::coupon_endpoint::get_by_id,
        endpoints::coupon_endpoint::add,
        endpoints::coupon_endpoint::update,
        endpoints::coupon_redemption_endpoint::list_all,
        endpoints::coupon_redemption_endpoint::paged,
        endpoints::coupon_redemption_endpoint::get_by_id,
        endpoints::coupon_redemption_endpoint::add,
        endpoints::cart_endpoint::list_all,
        endpoints::cart_endpoint::paged,
        endpoints::cart_endpoint::get_by_id,
        endpoints::cart_endpoint::add,
        endpoints::cart_endpoint::update,
        endpoints::cart_item_endpoint::list_all,
        endpoints::cart_item_endpoint::paged,
        endpoints::cart_item_endpoint::get_by_id,
        endpoints::cart_item_endpoint::add,
        endpoints::cart_item_endpoint::update,
        endpoints::orders_endpoint::list_all,
        endpoints::orders_endpoint::create_purchase,
        endpoints::orders_endpoint::purchases_paged,
        endpoints::orders_endpoint::get_purchase,
        endpoints::orders_endpoint::payments,
        endpoints::orders_endpoint::history,
        endpoints::orders_endpoint::transactions,
        endpoints::orders_endpoint::paged,
        endpoints::orders_endpoint::get_by_id,
        endpoints::product_category_endpoint::list_all,
        endpoints::product_category_endpoint::paged,
        endpoints::product_category_endpoint::get_by_id,
        endpoints::product_category_endpoint::by_product,
        endpoints::product_category_endpoint::by_category,
        endpoints::product_category_endpoint::add,
        endpoints::product_category_endpoint::update,
        endpoints::product_category_endpoint::delete,
        endpoints::sku_attribute_endpoint::list_all,
        endpoints::sku_attribute_endpoint::paged,
        endpoints::sku_attribute_endpoint::get_by_id,
        endpoints::sku_attribute_endpoint::by_sku,
        endpoints::sku_attribute_endpoint::by_product,
        endpoints::sku_attribute_endpoint::add,
        endpoints::sku_attribute_endpoint::update,
        endpoints::sku_attribute_endpoint::delete,
        endpoints::sku_stock_endpoint::list_all,
        endpoints::sku_stock_endpoint::paged,
        endpoints::sku_stock_endpoint::get_by_id,
        endpoints::sku_stock_endpoint::by_sku,
        endpoints::sku_stock_endpoint::add,
        endpoints::sku_stock_endpoint::update,
        endpoints::sku_stock_endpoint::delete,
        endpoints::web_store_endpoint::products,
        endpoints::web_store_endpoint::query_products,
        endpoints::web_store_endpoint::product_detail,
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
            endpoints::json::catalog_json::ProductAttributeJson,
            endpoints::json::catalog_json::SkuJson,
            endpoints::json::catalog_json::CategoryInputJson,
            endpoints::json::catalog_json::ProductInputJson,
            endpoints::json::catalog_json::SkuInputJson,
            endpoints::json::product_category_json::ProductCategoryJson,
            endpoints::json::product_category_json::ProductCategoryInputJson,
            endpoints::json::sku_attribute_json::SkuAttributeValueJson,
            endpoints::json::sku_attribute_json::SkuAttributeValueInputJson,
            endpoints::json::sku_stock_json::SkuStockJson,
            endpoints::json::sku_stock_json::SkuStockInputJson,
            endpoints::json::product_json::ProductJson,
            endpoints::json::product_image_json::ProductImagePresignItemRequest,
            endpoints::json::product_image_json::ProductImagePresignBatchRequest,
            endpoints::json::product_image_json::ProductImagePresignItemResponse,
            endpoints::json::product_image_json::ProductImageJson,
            endpoints::json::shipping_rate_json::ShippingRateJson,
            endpoints::json::shipping_rate_json::ShippingRateInputJson,
            endpoints::json::customer_json::CustomerJson,
            endpoints::json::customer_json::CustomerAddressJson,
            endpoints::json::person_json::PersonJson,
            endpoints::json::person_json::PersonInputJson,
            endpoints::json::person_json::PersonAddressJson,
            endpoints::json::person_json::PersonAddressInputJson,
            endpoints::json::person_json::AvatarPresignRequest,
            endpoints::json::person_json::AvatarPresignResponse,
            endpoints::json::person_json::ResourceProfileJson,
            endpoints::json::tax_rule_json::TaxRuleJson,
            endpoints::json::tax_rule_json::TaxRuleInputJson,
            endpoints::json::campaign_json::CampaignJson,
            endpoints::json::campaign_json::CampaignInputJson,
            endpoints::json::campaign_json::CampaignTargetJson,
            endpoints::json::campaign_json::CampaignTargetInputJson,
            endpoints::json::coupon_json::CouponJson,
            endpoints::json::coupon_json::CouponInputJson,
            endpoints::json::coupon_json::CouponRedemptionJson,
            endpoints::json::coupon_json::CouponRedemptionInputJson,
            endpoints::json::cart_json::CartJson,
            endpoints::json::cart_json::CartInputJson,
            endpoints::json::cart_json::CartItemJson,
            endpoints::json::cart_json::CartItemInputJson,
            endpoints::json::orders_json::OrdersJson,
            endpoints::json::orders_json::PurchaseJson,
            endpoints::json::orders_json::PaymentJson,
            endpoints::json::orders_json::PaymentAllocationJson,
            endpoints::json::orders_json::OrderAddressJson,
            endpoints::json::orders_json::CreditCardDetailsJson,
            endpoints::json::orders_json::PaymentTransactionJson,
            endpoints::json::orders_json::OrderDetailJson,
            endpoints::json::orders_json::PaymentDetailJson,
            endpoints::json::orders_json::PurchaseDetailJson,
            endpoints::json::orders_json::CreatePurchaseInputJson,
            endpoints::json::orders_json::PurchaseItemInputJson,
            endpoints::json::orders_json::AddressInputJson,
            endpoints::json::orders_json::OrderItemJson,
            endpoints::json::orders_json::OrderStatusHistoryJson,
            endpoints::json::web_store_json::WebStoreProductJson,
            endpoints::json::web_store_json::WebStoreProductDetailJson,
            endpoints::json::web_store_json::WebStoreSellerJson,
            endpoints::json::web_store_json::WebStoreImageJson,
            endpoints::json::web_store_json::WebStoreSkuJson,
            endpoints::json::web_store_json::WebStoreSkuAttributeValueJson,
            endpoints::json::web_store_json::WebStoreAttributeJson,
        ),
    ),
    tags(
        (name = "Novgorod", description = "REST API for Novgorod"),
        (name = "Tenant", description = "Tenant management endpoints"),
        (name = "User", description = "User management endpoints"),
        (name = "Province", description = "Province endpoints"),
        (name = "City", description = "City endpoints"),
        (name = "Category", description = "Product category endpoints"),
        (name = "Product", description = "Product catalog endpoints"),
        (name = "ProductCategory", description = "Product category assignment endpoints"),
        (name = "Sku", description = "SKU management endpoints"),
        (name = "SkuAttribute", description = "SKU attribute assignment endpoints"),
        (name = "SkuStock", description = "SKU inventory stock endpoints"),
        (name = "CatalogAttribute", description = "Catalog attribute endpoints"),
        (name = "ProductImage", description = "Product image endpoints"),
        (name = "ShippingRate", description = "Shipping rate endpoints"),
        (name = "Customer", description = "Customer endpoints"),
        (name = "CustomerAddress", description = "Customer address endpoints"),
        (name = "Person", description = "Person endpoints"),
        (name = "PersonAddress", description = "Person address endpoints"),
        (name = "Resource", description = "User profile and resource endpoints"),
        (name = "TaxRule", description = "Tax rule endpoints"),
        (name = "Campaign", description = "Marketing campaign endpoints"),
        (name = "CampaignTarget", description = "Campaign target endpoints"),
        (name = "Coupon", description = "Coupon endpoints"),
        (name = "CouponRedemption", description = "Coupon redemption endpoints"),
        (name = "Cart", description = "Shopping cart endpoints"),
        (name = "CartItem", description = "Cart item endpoints"),
        (name = "Orders", description = "Order management endpoints"),
        (name = "OrderItem", description = "Order item endpoints"),
        (name = "OrderStatusHistory", description = "Order status history endpoints"),
        (name = "WebStore", description = "Web store endpoints"),
    )
)]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub conn: Arc<DatabaseConnection>,
    pub storage: Arc<business::gateway::storage_gateway::StorageGateway>,
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

    let storage = Arc::new(business::gateway::storage_gateway::StorageGateway::from_env().await);
    crate::infrastructure::sqs_consumer::spawn_sqs_consumer(connection.clone(), storage.clone());

    let state = AppState {
        conn: Arc::new(connection),
        storage,
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
            header::HeaderName::from_static("idempotency-key"),
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
        .merge(customer_routes(state.clone()))
        .merge(person_routes(state.clone()))
        .merge(order_routes(state.clone()))
        .merge(cart_routes(state.clone()))
        .merge(marketing_routes(state.clone()))
        .merge(shipping_tax_routes(state.clone()))
        .nest("/tenant", tenant_routes(state.clone()))
        .nest("/user", user_routes(state.clone()))
        .nest("/api/public", web_store_routes())
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
