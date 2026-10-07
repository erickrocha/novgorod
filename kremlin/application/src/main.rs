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
use axum::routing::{get, post};
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
        endpoints::checkout_quote_endpoint::quote,
        endpoints::checkout_quote_endpoint::select_shipping,
        endpoints::shipping_settings_endpoint::get,
        endpoints::shipping_settings_endpoint::put,
        endpoints::payment_settings_endpoint::get,
        endpoints::payment_settings_endpoint::put,
        endpoints::tenant_endpoint::add,
        endpoints::tenant_endpoint::get_by_id,
        endpoints::tenant_endpoint::get_by_uuid,
        endpoints::tenant_endpoint::list_all,
        endpoints::tenant_endpoint::update,
        endpoints::tenant_endpoint::set_listing,
        endpoints::tenant_endpoint::get_listing,
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
        endpoints::shipping_rate_endpoint::delete,
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
        endpoints::warehouse_endpoint::list_all,
        endpoints::warehouse_endpoint::paged,
        endpoints::warehouse_endpoint::get_by_id,
        endpoints::warehouse_endpoint::add,
        endpoints::warehouse_endpoint::update,
        endpoints::warehouse_endpoint::delete,
        endpoints::web_store_endpoint::products,
        endpoints::web_store_endpoint::query_products,
        endpoints::web_store_endpoint::product_detail,
    ),
    components(
        schemas(
            endpoints::json::warehouse_json::WarehouseJson,
            endpoints::json::warehouse_json::WarehouseInputJson,
            endpoints::json::user_json::UserJson,
            endpoints::json::login_request::LoginRequest,
            endpoints::json::change_password_request::ChangePasswordRequest,
            endpoints::json::refresh_token_request::RefreshTokenRequest,
            endpoints::json::access_token_json::AccessTokenJson,
            endpoints::json::tenant_json::TenantJson,
            endpoints::tenant_endpoint::ListingJson,
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
            endpoints::json::orders_json::OrderItemJson,
            endpoints::json::orders_json::OrderStatusHistoryJson,
            endpoints::json::web_store_json::WebStoreProductJson,
            endpoints::json::web_store_json::WebStoreProductDetailJson,
            endpoints::json::web_store_json::WebStoreSellerJson,
            endpoints::json::web_store_json::WebStoreImageJson,
            endpoints::json::web_store_json::WebStoreSkuJson,
            endpoints::json::web_store_json::WebStoreSkuAttributeValueJson,
            endpoints::json::web_store_json::WebStoreAttributeJson,
            endpoints::payment_settings_endpoint::PaymentSettingsResponse,
            endpoints::payment_settings_endpoint::PaymentSettingsUpdate,
            endpoints::payment_settings_endpoint::PaymentCredentialsUpdate,
            endpoints::payment_settings_endpoint::MercadoPagoCredentialsUpdate,
            endpoints::payment_settings_endpoint::PagSeguroCredentialsUpdate,
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
    pub shipping: Arc<dyn business::gateway::shipping_provider_gateway::ShippingProviderGateway>,
    pub shipping_keys: Arc<infrastructure::shipping_credentials::ShippingKeyRing>,
    pub payment_keys: Arc<infrastructure::payment_credentials::PaymentKeyRing>,
    /// Shared secret nginx sends as `x-gateway-token` (GATEWAY_TOKEN); unset => every proof is invalid.
    pub gateway_token: Option<Arc<str>>,
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

    let shipping_keys = Arc::new(
        infrastructure::shipping_credentials::ShippingKeyRing::from_env()
            .map_err(anyhow::Error::msg)?,
    );
    let payment_keys = Arc::new(
        infrastructure::payment_credentials::PaymentKeyRing::from_env()
            .map_err(anyhow::Error::msg)?,
    );
    let shipping = Arc::new(
        infrastructure::correios::Correios::new(connection.clone(), shipping_keys.clone())
            .map_err(anyhow::Error::msg)?,
    );
    let state = AppState {
        conn: Arc::new(connection),
        storage,
        shipping,
        shipping_keys,
        payment_keys,
        gateway_token: env::var("GATEWAY_TOKEN")
            .ok()
            .filter(|token| !token.is_empty())
            .map(Arc::from),
    };
    crate::endpoints::checkout_payment_endpoint::spawn_payment_reconciliation(state.clone());
    crate::endpoints::checkout_payment_endpoint::spawn_stock_expiry(state.clone());

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
            header::HeaderName::from_static("x-tenant-id"),
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
        .route(
            "/webhooks/mercado-pago",
            post(endpoints::checkout_payment_endpoint::webhook),
        )
        .route(
            "/webhooks/pagseguro",
            post(endpoints::checkout_payment_endpoint::pagseguro_webhook),
        )
        .merge(cart_routes(state.clone()))
        .merge(marketing_routes(state.clone()))
        .merge(shipping_tax_routes(state.clone()))
        .nest("/tenant", tenant_routes(state.clone()))
        .merge(routes::shipping_settings_routes::shipping_settings_routes(
            state.clone(),
        ))
        .merge(routes::payment_settings_routes::payment_settings_routes(
            state.clone(),
        ))
        .nest("/user", user_routes(state.clone()))
        .nest("/api/public", web_store_routes())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::commons::tenant_context::tenant_context,
        ))
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

#[cfg(test)]
mod shipping_api_tests {
    use super::*;
    use business::domain::enums::Role;

    #[test]
    fn customer_token_payload_has_no_tenant_binding() {
        let payload = endpoints::json::access_token_json::AccessTokenJson {
            access_token: "test-token".into(),
            token_type: "Bearer".into(),
            expire_in: 3600,
            refresh_token: None,
            email: "customer@example.test".into(),
            uuid: "customer-uuid".into(),
            name: "Customer".into(),
            user_id: 7,
            role: Role::Customer,
            tenant_id: None,
            first_login: false,
        };
        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json["role"], "Customer");
        assert!(json["tenantId"].is_null());
    }

    #[test]
    fn public_catalog_prices_serialize_as_integer_cents() {
        let product = endpoints::json::web_store_json::WebStoreProductJson {
            id: 1,
            uuid: "product-uuid".into(),
            name: "Test Product".into(),
            slug: "test-product".into(),
            description: None,
            brand: None,
            price_cents: Some(1099),
            compare_at_price_cents: Some(1299),
            primary_image_url: None,
            primary_image_alt: None,
            category_slugs: vec![],
            rating: None,
            review_count: None,
            is_featured: false,
            is_new: false,
            seller: None,
        };
        let json = serde_json::to_value(product).unwrap();
        assert_eq!(json["priceCents"].as_i64(), Some(1099));
        assert_eq!(json["compareAtPriceCents"].as_i64(), Some(1299));
        assert!(json["price"].is_null());
    }

    #[test]
    fn c001_openapi_baseline_exposes_route_and_public_security_gaps() {
        let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let documented_products = [
            ("/api/public/products", "get"),
            ("/api/public/products/query", "get"),
            ("/api/public/products/{slug}", "get"),
        ];
        for (path, method) in documented_products {
            assert!(
                doc["paths"][path][method].is_object(),
                "missing OpenAPI operation {method} {path}"
            );
        }

        for path in ["/api/public/products/query", "/api/public/products/{slug}"] {
            assert_eq!(
                doc["paths"][path]["get"]["security"][0]["bearer_auth"].is_array(),
                true,
                "record the current public-route bearer annotation on {path}"
            );
        }

        for path in [
            "/login",
            "/signup",
            "/customers/me",
            "/customers/me/tax-id",
            "/api/public/categories",
            "/api/public/categories/paged",
            "/api/public/skus",
            "/api/public/skus/paged",
            "/api/public/catalog-attributes",
            "/api/public/product-categories",
            "/api/public/sku-attributes",
            "/api/public/sku-attribute-values",
            "/api/public/sku-stocks",
        ] {
            assert!(
                doc["paths"][path].is_null(),
                "record current undocumented route {path}"
            );
        }
    }

    #[test]
    fn shipping_endpoints_and_write_only_secrets_are_documented() {
        let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
        assert!(doc["paths"]["/checkout/quotes"]["post"].is_object());
        assert!(doc["paths"]["/checkout/quotes/{id}/shipping-selection"]["post"].is_object());
        assert!(doc["paths"]["/tenants/{tenantId}/shipping-settings"]["put"].is_object());
        assert_eq!(
            doc["components"]["schemas"]["CredentialsUpdate"]["properties"]["apiAccessCode"]["writeOnly"],
            true
        );
    }
}

#[cfg(test)]
#[path = "../tests/unit/infrastructure/shipping_http_test.rs"]
mod shipping_http_tests;

#[cfg(test)]
mod c001_auth_middleware_tests {
    use super::*;
    use axum::{
        Extension, Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::{get, post},
    };
    use business::{
        domain::{enums::Role, shipping::ShippingOption, user::User},
        gateway::shipping_provider_gateway::{
            ShippingProviderError, ShippingProviderGateway, ShippingRequest,
        },
        sea_orm::{DatabaseBackend, DbConn, MockDatabase, MockExecResult, Value},
    };
    use chrono::Utc;
    use entity::user_entity;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use crate::authentication::authentication_middleware::authentication;
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use tower::ServiceExt;

    const TEST_SECRET: &str = "c001-test-secret-long-enough-for-hs512-auth-middleware-tests";

    struct UnusedShippingProvider;

    #[business::sea_orm::prelude::async_trait::async_trait]
    impl ShippingProviderGateway for UnusedShippingProvider {
        async fn quote(
            &self,
            _request: ShippingRequest,
        ) -> Result<Vec<ShippingOption>, ShippingProviderError> {
            Err(ShippingProviderError::Unavailable)
        }
    }

    fn state(db: DbConn) -> AppState {
        let storage_client = aws_sdk_s3::Client::from_conf(
            aws_sdk_s3::Config::builder()
                .behavior_version(aws_config::BehaviorVersion::latest())
                .build(),
        );
        AppState {
            conn: Arc::new(db),
            storage: Arc::new(business::gateway::storage_gateway::StorageGateway::new(
                "unused-test-bucket".into(),
                "http://localhost.invalid".into(),
                storage_client,
            )),
            shipping: Arc::new(UnusedShippingProvider),
            shipping_keys: Arc::new(infrastructure::shipping_credentials::ShippingKeyRing::default()),
            payment_keys: Arc::new(infrastructure::payment_credentials::PaymentKeyRing::default()),
            gateway_token: None,
        }
    }

    fn user_model() -> user_entity::Model {
        user_model_for(Role::Customer, None)
    }

    fn user_model_for(role: Role, tenant_id: Option<i64>) -> user_entity::Model {
        let now = Utc::now().naive_utc();
        user_entity::Model {
            id: 42,
            uuid: uuid::Uuid::new_v4(),
            name: Some("C001 role matrix".into()),
            email: "c001@example.test".into(),
            password: String::new(),
            first_login: false,
            enabled: true,
            tenant_id,
            role: role.to_string(),
            blocked_reason: None,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        }
    }

    fn customer_model(id: i64, user_id: i64) -> entity::customer_entity::Model {
        let now = Utc::now().naive_utc();
        entity::customer_entity::Model {
            id,
            uuid: uuid::Uuid::new_v4(),
            tenant_id: None,
            user_id: Some(user_id),
            name: "C001 customer".into(),
            email: "c001@example.test".into(),
            cpf: None,
            phone: None,
            marketing_consent: false,
            consent_at: None,
            active: true,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        }
    }

    fn customer_address_model(
        id: i64,
        tenant_id: Option<i64>,
        customer_id: i64,
    ) -> entity::customer_address_entity::Model {
        let now = Utc::now().naive_utc();
        entity::customer_address_entity::Model {
            id,
            uuid: uuid::Uuid::new_v4(),
            tenant_id,
            customer_id,
            label: Some("Home".into()),
            recipient: "C001 customer".into(),
            address_line1: Some("Test street 1".into()),
            address_line2: None,
            locality: Some("Test city".into()),
            administrative_area: Some("SP".into()),
            postal_code: Some("01001000".into()),
            country_code: Some("BR".into()),
            is_default: true,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        }
    }

    fn person_model(id: i64, user_id: i64, tenant_id: Option<i64>) -> entity::person_entity::Model {
        let now = Utc::now().naive_utc();
        entity::person_entity::Model {
            id,
            uuid: uuid::Uuid::new_v4(),
            tenant_id,
            user_id,
            first_name: "Created user".into(),
            surname: None,
            date_of_birth: None,
            gender: None,
            avatar: None,
            phone: None,
            email: None,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        }
    }

    fn person_address_model(id: i64, person_id: i64, tenant_id: Option<i64>) -> entity::person_address_entity::Model {
        let now = Utc::now().naive_utc();
        entity::person_address_entity::Model {
            id,
            uuid: uuid::Uuid::new_v4(),
            tenant_id,
            person_id,
            address_line1: Some("Test street 1".into()),
            address_line2: None,
            locality: Some("Test city".into()),
            administrative_area: Some("SP".into()),
            postal_code: Some("01001000".into()),
            country_code: Some("BR".into()),
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        }
    }

    fn tenant_model(id: i64) -> entity::tenant_entity::Model {
        let now = Utc::now().naive_utc();
        entity::tenant_entity::Model {
            id,
            uuid: uuid::Uuid::new_v4(),
            business_name: format!("Tenant {id}"),
            company_name: None,
            tax_id: format!("tax-{id}"),
            email: Some(format!("tenant{id}@example.test")),
            phone: None,
            web_site: None,
            address_line1: None,
            address_line2: None,
            locality: None,
            administrative_area: None,
            postal_code: None,
            country_code: Some("BR".into()),
            listed: true,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        }
    }

    fn token(expires_at: i64) -> String {
        token_for(Role::Customer, None, expires_at)
    }

    fn token_for(role: Role, tenant_id: Option<i64>, expires_at: i64) -> String {
        let claims = business::domain::access_token::Claims {
            sub: "c001@example.test".into(),
            exp: expires_at,
            uuid: "c001-user".into(),
            name: "C001 role matrix".into(),
            user_id: 42,
            role,
            tenant_id,
        };
        encode(
            &Header::new(Algorithm::HS512),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .unwrap()
    }

    fn request(method: &str, path: &str, authorization: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(value) = authorization {
            builder = builder.header("authorization", value);
        }
        builder.body(Body::empty()).unwrap()
    }

    fn json_request(method: &str, path: &str, authorization: &str, body: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(path)
            .header("authorization", authorization)
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    #[tokio::test]
    async fn authentication_middleware_rejects_expired_bearer_tokens() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model()], vec![user_model()]])
            .into_connection();
        let app = Router::new()
            .route(
                "/protected",
                get(|Extension(user): Extension<User>| async move { user.role.to_string() }),
            )
            .route("/login", post(|| async { StatusCode::OK }))
            .route("/signup", post(|| async { StatusCode::OK }))
            .route_layer(middleware::from_fn_with_state(state(db), authentication));

        assert_eq!(
            app.clone()
                .oneshot(request("GET", "/protected", None))
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            app.clone()
                .oneshot(request("GET", "/protected", Some("Basic not-a-bearer")))
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            app.clone()
                .oneshot(request("GET", "/protected", Some("Bearer malformed-token")))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            app.clone()
                .oneshot(request(
                    "GET",
                    "/protected",
                    Some(&format!("Bearer {}", token(Utc::now().timestamp() - 120))),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            app.clone()
                .oneshot(request(
                    "GET",
                    "/protected",
                    Some(&format!("Bearer {}", token(Utc::now().timestamp() + 3600))),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        for path in ["/login", "/signup"] {
            assert_eq!(
                app.clone()
                    .oneshot(request("POST", path, None))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::OK,
                "{path} remains public"
            );
        }
    }

    #[tokio::test]
    async fn customer_list_route_scopes_query_for_authenticated_roles() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, "unrestricted"),
            (Role::TenantOwner, Some(7), "tenant"),
            (Role::TenantOwner, Some(8), "tenant"),
            (Role::TenantUser, Some(7), "tenant"),
            (Role::Customer, None, "deny"),
        ];

        for (role, tenant_id, expected_scope) in cases {
            for (path, expected_status) in [
                ("/customers", StatusCode::OK),
                ("/customers/99", StatusCode::NOT_FOUND),
            ] {
                let db = MockDatabase::new(DatabaseBackend::Postgres)
                    .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                    .append_query_results([Vec::<entity::customer_entity::Model>::new()])
                    .into_connection();
                let db_for_log = db.clone();
                let app_state = state(db);
                let app = crate::routes::customer_routes::customer_routes(app_state.clone())
                    .with_state(app_state);
                let response = app
                    .oneshot(request(
                        "GET",
                        path,
                        Some(&format!(
                            "Bearer {}",
                            token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                        )),
                    ))
                    .await
                    .unwrap();

                assert_eq!(response.status(), expected_status, "role {role:?}, {path}");
                let _body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let transaction_log = db_for_log.into_transaction_log();
                let customer_query = transaction_log
                    .iter()
                    .flat_map(|transaction| transaction.statements())
                    .find(|statement| statement.sql.contains("FROM \"customer\""))
                    .expect("customer query should be executed");
                let where_clause = customer_query
                    .sql
                    .split_once(" WHERE ")
                    .map(|(_, clause)| clause)
                    .unwrap_or_default();

                match expected_scope {
                    "unrestricted" => {
                        if path == "/customers" {
                            assert!(
                                where_clause.is_empty(),
                                "SysAdmin collection query should be unrestricted: {}",
                                customer_query.sql
                            );
                        } else {
                            assert!(
                                where_clause.contains("id"),
                                "SysAdmin resource query should filter by requested ID: {}",
                                customer_query.sql
                            );
                            assert!(
                                !where_clause.contains("tenant_id"),
                                "SysAdmin resource query should not be tenant-filtered: {}",
                                customer_query.sql
                            );
                        }
                    }
                    "tenant" => {
                        assert!(
                            where_clause.contains("tenant_id"),
                            "tenant role query should include its tenant filter: {}",
                            customer_query.sql
                        );
                        if path == "/customers/99" {
                            assert!(
                                where_clause.contains("id"),
                                "tenant resource query should also filter by requested ID: {}",
                                customer_query.sql
                            );
                        }
                    }
                    "deny" => assert!(
                        where_clause.contains("1 = 0"),
                        "tenantless Customer query should deny tenant-scoped data: {}",
                        customer_query.sql
                    ),
                    _ => unreachable!("unknown expected scope"),
                }
            }
        }
    }

    #[tokio::test]
    async fn customer_paged_route_scopes_queries_by_role() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, "unrestricted"),
            (Role::TenantOwner, Some(7), "tenant"),
            (Role::TenantUser, Some(7), "tenant"),
            (Role::Customer, None, "no_query"),
        ];
        let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

        for (role, tenant_id, expected_scope) in cases {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![count_row.clone()]])
                .append_query_results([Vec::<entity::customer_entity::Model>::new()])
                .into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = crate::routes::customer_routes::customer_routes(app_state.clone())
                .with_state(app_state);
            let response = app
                .oneshot(request(
                    "GET",
                    "/customers/paged",
                    Some(&format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    )),
                ))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK, "GET paged customers as {role:?}");
            let customer_queries: Vec<_> = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .filter(|statement| statement.sql.contains("FROM \"customer\""))
                .collect();
            match expected_scope {
                "unrestricted" => {
                    let list_query = customer_queries.last().expect("SysAdmin paged query");
                    let where_clause = list_query
                        .sql
                        .split_once(" WHERE ")
                        .map(|(_, clause)| clause)
                        .unwrap_or_default();
                    assert!(!where_clause.contains("tenant_id"), "{}", list_query.sql);
                }
                "tenant" => {
                    let list_query = customer_queries.last().expect("tenant paged query");
                    assert!(list_query.sql.contains("tenant_id"), "{}", list_query.sql);
                }
                "no_query" => assert!(customer_queries.is_empty(), "Customer tenantless query was issued"),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }

    #[tokio::test]
    async fn customer_address_collection_routes_scope_queries_by_role() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, "unrestricted"),
            (Role::TenantOwner, Some(7), "tenant"),
            (Role::TenantUser, Some(7), "tenant"),
            (Role::Customer, None, "no_query"),
        ];
        let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

        for (role, tenant_id, expected_scope) in cases {
            for path in ["/customer-addresses", "/customer-addresses/paged"] {
                let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                    .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
                if path.ends_with("/paged") {
                    db = db.append_query_results([vec![count_row.clone()]]);
                }
                let db = db
                    .append_query_results([Vec::<entity::customer_address_entity::Model>::new()])
                    .into_connection();
                let db_for_log = db.clone();
                let app_state = state(db);
                let app = crate::routes::customer_routes::customer_routes(app_state.clone())
                    .with_state(app_state);
                let response = app
                    .oneshot(request(
                        "GET",
                        path,
                        Some(&format!(
                            "Bearer {}",
                            token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                        )),
                    ))
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::OK, "GET {path} as {role:?}");
                let address_queries: Vec<_> = db_for_log
                    .into_transaction_log()
                    .into_iter()
                    .flat_map(|transaction| transaction.statements().to_vec())
                    .filter(|statement| statement.sql.contains("FROM \"customer_address\""))
                    .collect();
                let path_scope = if role == Role::Customer && path == "/customer-addresses" {
                    "deny"
                } else {
                    expected_scope
                };
                match path_scope {
                    "unrestricted" => {
                        let list_query = address_queries.last().expect("SysAdmin address query");
                        let where_clause = list_query
                            .sql
                            .split_once(" WHERE ")
                            .map(|(_, clause)| clause)
                            .unwrap_or_default();
                        assert!(!where_clause.contains("tenant_id"), "{}", list_query.sql);
                    }
                    "tenant" => {
                        let list_query = address_queries.last().expect("tenant address query");
                        assert!(list_query.sql.contains("tenant_id"), "{}", list_query.sql);
                    }
                    "deny" => {
                        let list_query = address_queries.last().expect("Customer address query");
                        let where_clause = list_query
                            .sql
                            .split_once(" WHERE ")
                            .map(|(_, clause)| clause)
                            .unwrap_or_default();
                        assert!(where_clause.contains("1 = 0"), "{}", list_query.sql);
                    }
                    "no_query" => assert!(address_queries.is_empty(), "Customer tenantless query was issued"),
                    _ => unreachable!("unknown expected scope"),
                }
            }
        }
    }

    #[tokio::test]
    async fn customer_address_create_route_stamps_authenticated_tenant_scope() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, Some(8)),
            (Role::TenantOwner, Some(7), Some(7)),
            (Role::TenantUser, Some(7), Some(7)),
            (Role::Customer, None, None),
        ];
        let payload = r#"{"tenantId":8,"customerId":19,"recipient":"C001 customer","isDefault":true}"#;

        for (role, tenant_id, expected_tenant_id) in cases {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![customer_address_model(90, expected_tenant_id, 19)]])
                .into_connection();
            let app_state = state(db);
            let app = crate::routes::customer_routes::customer_routes(app_state.clone())
                .with_state(app_state);
            let response = app
                .oneshot(json_request(
                    "POST",
                    "/customer-addresses",
                    &format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    ),
                    payload,
                ))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::CREATED, "POST address as {role:?}");
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let address: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(
                address["tenantId"],
                expected_tenant_id.map_or(serde_json::Value::Null, serde_json::Value::from),
                "address tenant must follow the authenticated scope for {role:?}"
            );
        }
    }

    #[tokio::test]
    async fn customer_address_update_route_enforces_tenant_scope() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, true, StatusCode::OK),
            (Role::TenantOwner, Some(7), true, StatusCode::OK),
            (Role::TenantOwner, Some(8), false, StatusCode::NOT_FOUND),
            (Role::TenantUser, Some(7), true, StatusCode::OK),
            (Role::Customer, None, false, StatusCode::NOT_FOUND),
        ];
        let update_body = r#"{"tenantId":7,"customerId":19,"recipient":"Updated C001 customer","isDefault":true}"#;

        for (role, tenant_id, address_in_scope, expected_status) in cases {
            let address = customer_address_model(90, Some(7), 19);
            let mut query_results = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([if address_in_scope {
                    vec![address.clone()]
                } else {
                    Vec::new()
                }]);
            if address_in_scope {
                query_results = query_results.append_query_results([vec![customer_address_model(
                    90,
                    Some(7),
                    19,
                )]]);
            }
            let db = query_results.into_connection();
            let app_state = state(db);
            let app = crate::routes::customer_routes::customer_routes(app_state.clone())
                .with_state(app_state);
            let response = app
                .oneshot(json_request(
                    "PUT",
                    "/customer-addresses/90",
                    &format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    ),
                    update_body,
                ))
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                expected_status,
                "PUT T7 customer address as {role:?} for tenant {tenant_id:?}"
            );
        }
    }

    #[tokio::test]
    async fn payment_settings_routes_characterize_role_and_tenant_matrix() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, StatusCode::OK, StatusCode::NOT_FOUND),
            (Role::TenantOwner, Some(7), StatusCode::OK, StatusCode::NOT_FOUND),
            (Role::TenantOwner, Some(8), StatusCode::FORBIDDEN, StatusCode::FORBIDDEN),
            (Role::TenantUser, Some(7), StatusCode::FORBIDDEN, StatusCode::FORBIDDEN),
            (Role::Customer, None, StatusCode::FORBIDDEN, StatusCode::FORBIDDEN),
        ];

        for (role, tenant_id, expected_get, expected_put) in cases {
            for (method, expected_status) in [
                ("GET", expected_get),
                ("PUT", expected_put),
            ] {
                let db = MockDatabase::new(DatabaseBackend::Postgres)
                    .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                    .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
                    .into_connection();
                let app_state = state(db);
                let app = crate::routes::payment_settings_routes::payment_settings_routes(
                    app_state.clone(),
                )
                .with_state(app_state);
                let authorization = format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                );
                let request = if method == "GET" {
                    request(method, "/tenants/7/payment-settings", Some(&authorization))
                } else {
                    json_request(
                        method,
                        "/tenants/7/payment-settings",
                        &authorization,
                        r#"{"provider":"mercado_pago"}"#,
                    )
                };
                let response = app.oneshot(request).await.unwrap();

                assert_eq!(
                    response.status(),
                    expected_status,
                    "{method} payment settings as {role:?} for tenant {tenant_id:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn shipping_settings_routes_characterize_role_and_tenant_matrix() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, StatusCode::NOT_FOUND, StatusCode::NOT_FOUND),
            (Role::TenantOwner, Some(7), StatusCode::NOT_FOUND, StatusCode::NOT_FOUND),
            (Role::TenantOwner, Some(8), StatusCode::FORBIDDEN, StatusCode::FORBIDDEN),
            (Role::TenantUser, Some(7), StatusCode::FORBIDDEN, StatusCode::FORBIDDEN),
            (Role::Customer, None, StatusCode::FORBIDDEN, StatusCode::FORBIDDEN),
        ];
        let update_body = serde_json::json!({
            "configuration": business::domain::shipping::ShippingConfig::default()
        })
        .to_string();

        for (role, tenant_id, expected_get, expected_put) in cases {
            for (method, expected_status) in [
                ("GET", expected_get),
                ("PUT", expected_put),
            ] {
                let db = MockDatabase::new(DatabaseBackend::Postgres)
                    .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                    .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
                    .into_connection();
                let app_state = state(db);
                let app = crate::routes::shipping_settings_routes::shipping_settings_routes(
                    app_state.clone(),
                )
                .with_state(app_state);
                let authorization = format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                );
                let request = if method == "GET" {
                    request(method, "/tenants/7/shipping-settings", Some(&authorization))
                } else {
                    json_request(
                        method,
                        "/tenants/7/shipping-settings",
                        &authorization,
                        &update_body,
                    )
                };
                let response = app.oneshot(request).await.unwrap();

                assert_eq!(
                    response.status(),
                    expected_status,
                    "{method} shipping settings as {role:?} for tenant {tenant_id:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn tenant_listing_route_characterizes_role_and_tenant_matrix() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, StatusCode::OK),
            (Role::TenantOwner, Some(7), StatusCode::OK),
            (Role::TenantOwner, Some(8), StatusCode::NOT_FOUND),
            (Role::TenantUser, Some(7), StatusCode::FORBIDDEN),
            (Role::Customer, None, StatusCode::NOT_FOUND),
        ];
        let listing_row = BTreeMap::from([(
            "listed".to_string(),
            Value::Bool(Some(true)),
        )]);

        for (role, tenant_id, expected_status) in cases {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![listing_row.clone()]])
                .into_connection();
            let app_state = state(db);
            let app = Router::new()
                .nest(
                    "/tenant",
                    crate::routes::tenant_routes::tenant_routes(app_state.clone()),
                )
                .with_state(app_state);
            let response = app
                .oneshot(request(
                    "GET",
                    "/tenant/7/listing",
                    Some(&format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    )),
                ))
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                expected_status,
                "GET tenant listing as {role:?} for tenant {tenant_id:?}"
            );
        }
    }

    #[tokio::test]
    async fn tenant_by_id_route_characterizes_role_and_tenant_access() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, StatusCode::OK),
            (Role::TenantOwner, Some(7), StatusCode::OK),
            (Role::TenantOwner, Some(8), StatusCode::NOT_FOUND),
            (Role::TenantUser, Some(7), StatusCode::OK),
            (Role::Customer, None, StatusCode::NOT_FOUND),
        ];

        for (role, tenant_id, expected_status) in cases {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![tenant_model(7)]])
                .into_connection();
            let app_state = state(db);
            let app = Router::new()
                .nest("/tenant", crate::routes::tenant_routes::tenant_routes(app_state.clone()))
                .with_state(app_state);
            let response = app
                .oneshot(request(
                    "GET",
                    "/tenant/7",
                    Some(&format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    )),
                ))
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                expected_status,
                "GET tenant 7 as {role:?} for tenant {tenant_id:?}"
            );
        }
    }

    #[tokio::test]
    async fn tenant_collection_routes_scope_by_role_and_tenant() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, "unrestricted"),
            (Role::TenantOwner, Some(7), "tenant"),
            (Role::TenantUser, Some(7), "tenant"),
            (Role::Customer, None, "no_query"),
        ];
        let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(2)))]);

        for (role, tenant_id, expected_scope) in cases {
            for path in ["/tenant", "/tenant/paged"] {
                let tenant_rows = match expected_scope {
                    "unrestricted" => vec![tenant_model(7), tenant_model(8)],
                    "tenant" => vec![tenant_model(7)],
                    _ => Vec::new(),
                };
                let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                    .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
                if path.ends_with("/paged") && expected_scope != "no_query" {
                    db = db.append_query_results([vec![count_row.clone()]]);
                }
                if expected_scope != "no_query" {
                    db = db.append_query_results([tenant_rows]);
                }
                let db = db.into_connection();
                let db_for_log = db.clone();
                let app_state = state(db);
                let app = Router::new()
                    .nest("/tenant", crate::routes::tenant_routes::tenant_routes(app_state.clone()))
                    .with_state(app_state);
                let response = app
                    .oneshot(request(
                        "GET",
                        path,
                        Some(&format!(
                            "Bearer {}",
                            token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                        )),
                    ))
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::OK, "GET {path} as {role:?}");
                let tenant_queries: Vec<_> = db_for_log
                    .into_transaction_log()
                    .into_iter()
                    .flat_map(|transaction| transaction.statements().to_vec())
                    .filter(|statement| statement.sql.contains("FROM \"tenant\""))
                    .collect();
                match expected_scope {
                    "unrestricted" => {
                        let list_query = tenant_queries.last().expect("SysAdmin tenant query");
                        assert!(
                            !list_query.sql.contains(" WHERE "),
                            "SysAdmin tenant query should be unrestricted: {}",
                            list_query.sql
                        );
                    }
                    "tenant" => {
                        let list_query = tenant_queries.last().expect("tenant-scoped query");
                        assert!(
                            list_query.sql.contains("\"tenant\".\"id\" ="),
                            "tenant-scoped query should filter on tenant ID: {}",
                            list_query.sql
                        );
                    }
                    "no_query" => assert!(tenant_queries.is_empty(), "tenantless Customer query was issued"),
                    _ => unreachable!("unknown expected scope"),
                }
            }
        }
    }

    #[tokio::test]
    async fn person_by_id_route_enforces_tenant_scope() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, true, StatusCode::OK),
            (Role::TenantOwner, Some(7), true, StatusCode::OK),
            (Role::TenantOwner, Some(8), false, StatusCode::NOT_FOUND),
            (Role::TenantUser, Some(7), true, StatusCode::OK),
            (Role::Customer, None, false, StatusCode::NOT_FOUND),
        ];

        for (role, tenant_id, person_in_scope, expected_status) in cases {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([if person_in_scope {
                    vec![person_model(99, 42, Some(7))]
                } else {
                    Vec::new()
                }])
                .into_connection();
            let app_state = state(db);
            let app = Router::new()
                .merge(crate::routes::person_routes::person_routes(app_state.clone()))
                .with_state(app_state);
            let response = app
                .oneshot(request(
                    "GET",
                    "/persons/99",
                    Some(&format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    )),
                ))
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                expected_status,
                "GET T7 person as {role:?} for tenant {tenant_id:?}"
            );
        }
    }

    #[tokio::test]
    async fn person_collection_routes_scope_queries_by_role() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, "unrestricted"),
            (Role::TenantOwner, Some(7), "tenant"),
            (Role::TenantUser, Some(7), "tenant"),
            (Role::Customer, None, "deny"),
        ];
        let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);
        let paths = [
            "/persons",
            "/persons/paged",
            "/person-addresses",
            "/person-addresses/paged",
            "/person-addresses/by-person/99",
        ];

        for (role, tenant_id, expected_scope) in cases {
            for path in paths {
                let is_paged = path.ends_with("/paged");
                let is_person = path.starts_with("/persons");
                let path_scope = if expected_scope == "deny" && is_paged {
                    "no_query"
                } else {
                    expected_scope
                };
                let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                    .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
                if is_paged && path_scope != "no_query" {
                    db = db.append_query_results([vec![count_row.clone()]]);
                }
                if path_scope != "no_query" {
                    if is_person {
                        let rows = match expected_scope {
                            "unrestricted" => vec![
                                person_model(101, 42, Some(7)),
                                person_model(102, 43, Some(8)),
                            ],
                            "tenant" => vec![person_model(101, 42, Some(7))],
                            "deny" => Vec::new(),
                            _ => unreachable!("unknown expected scope"),
                        };
                        db = db.append_query_results([rows]);
                    } else {
                        let rows = match expected_scope {
                            "unrestricted" => vec![
                                person_address_model(201, 101, Some(7)),
                                person_address_model(202, 102, Some(8)),
                            ],
                            "tenant" => vec![person_address_model(201, 101, Some(7))],
                            "deny" => Vec::new(),
                            _ => unreachable!("unknown expected scope"),
                        };
                        db = db.append_query_results([rows]);
                    }
                }
                let db = db.into_connection();
                let db_for_log = db.clone();
                let app_state = state(db);
                let app = crate::routes::person_routes::person_routes(app_state.clone())
                    .with_state(app_state);
                let response = app
                    .oneshot(request(
                        "GET",
                        path,
                        Some(&format!(
                            "Bearer {}",
                            token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                        )),
                    ))
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::OK, "GET {path} as {role:?}");
                let table = if is_person { "person\"" } else { "person_address\"" };
                let queries: Vec<_> = db_for_log
                    .into_transaction_log()
                    .into_iter()
                    .flat_map(|transaction| transaction.statements().to_vec())
                    .filter(|statement| statement.sql.contains(&format!("FROM \"{table}")))
                    .collect();
                match path_scope {
                    "unrestricted" => {
                        let list_query = queries.last().expect("SysAdmin collection query");
                        if path.contains("by-person") {
                            let where_clause = list_query
                                .sql
                                .split_once(" WHERE ")
                                .map(|(_, clause)| clause)
                                .unwrap_or_default();
                            assert!(
                                !where_clause.contains("tenant_id"),
                                "SysAdmin by-person query should not be tenant-filtered: {}",
                                list_query.sql
                            );
                        } else {
                            assert!(
                                !list_query.sql.contains(" WHERE "),
                                "SysAdmin query should be unrestricted: {}",
                                list_query.sql
                            );
                        }
                    }
                    "tenant" => {
                        let list_query = queries.last().expect("tenant collection query");
                        let where_clause = list_query
                            .sql
                            .split_once(" WHERE ")
                            .map(|(_, clause)| clause)
                            .unwrap_or_default();
                        assert!(
                            where_clause.contains("tenant_id"),
                            "tenant role query should filter by tenant: {}",
                            list_query.sql
                        );
                    }
                    "deny" => {
                        let list_query = queries.last().expect("tenantless collection query");
                        assert!(
                            list_query.sql.contains("1 = 0"),
                            "tenantless role query should be deny-all: {}",
                            list_query.sql
                        );
                    }
                    "no_query" => assert!(
                        queries.is_empty(),
                        "tenantless Customer paged route should return before querying"
                    ),
                    _ => unreachable!("unknown expected scope"),
                }
            }
        }
    }

    #[tokio::test]
    async fn person_and_address_by_id_routes_enforce_tenant_scope() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, true, "unrestricted", StatusCode::OK),
            (Role::TenantOwner, Some(7), true, "tenant", StatusCode::OK),
            (Role::TenantOwner, Some(8), false, "tenant", StatusCode::NOT_FOUND),
            (Role::TenantUser, Some(7), true, "tenant", StatusCode::OK),
            (Role::Customer, None, false, "deny", StatusCode::NOT_FOUND),
        ];

        for (role, tenant_id, resource_in_scope, expected_scope, expected_status) in cases {
            for (path, table) in [
                ("/persons/99", "person"),
                ("/person-addresses/88", "person_address"),
            ] {
                let db = if table == "person" {
                    let rows = if resource_in_scope {
                        vec![person_model(99, 42, Some(7))]
                    } else {
                        Vec::<entity::person_entity::Model>::new()
                    };
                    MockDatabase::new(DatabaseBackend::Postgres)
                        .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                        .append_query_results([rows])
                        .into_connection()
                } else {
                    let rows = if resource_in_scope {
                        vec![person_address_model(88, 99, Some(7))]
                    } else {
                        Vec::<entity::person_address_entity::Model>::new()
                    };
                    MockDatabase::new(DatabaseBackend::Postgres)
                        .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                        .append_query_results([rows])
                        .into_connection()
                };
                let db_for_log = db.clone();
                let app_state = state(db);
                let app = crate::routes::person_routes::person_routes(app_state.clone())
                    .with_state(app_state);
                let response = app
                    .oneshot(request(
                        "GET",
                        path,
                        Some(&format!(
                            "Bearer {}",
                            token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                        )),
                    ))
                    .await
                    .unwrap();

                assert_eq!(response.status(), expected_status, "GET {path} as {role:?}");
                let resource_query = db_for_log
                    .into_transaction_log()
                    .into_iter()
                    .flat_map(|transaction| transaction.statements().to_vec())
                    .filter(|statement| statement.sql.contains(&format!("FROM \"{table}\"")))
                    .last()
                    .expect("resource-by-ID query should execute");
                let where_clause = resource_query
                    .sql
                    .split_once(" WHERE ")
                    .map(|(_, clause)| clause)
                    .unwrap_or_default();

                match expected_scope {
                    "unrestricted" => assert!(
                        where_clause.contains("id") && !where_clause.contains("tenant_id"),
                        "SysAdmin lookup should filter by ID only: {}",
                        resource_query.sql
                    ),
                    "tenant" => assert!(
                        where_clause.contains("tenant_id") && where_clause.contains("id"),
                        "tenant lookup should filter by tenant and ID: {}",
                        resource_query.sql
                    ),
                    "deny" => assert!(
                        where_clause.contains("1 = 0") && where_clause.contains("id"),
                        "tenantless Customer lookup should be deny-all by ID: {}",
                        resource_query.sql
                    ),
                    _ => unreachable!("unknown expected scope"),
                }
            }
        }
    }

    #[tokio::test]
    async fn tenant_listing_update_route_characterizes_role_and_tenant_matrix() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, StatusCode::OK),
            (Role::TenantOwner, Some(7), StatusCode::OK),
            (Role::TenantOwner, Some(8), StatusCode::NOT_FOUND),
            (Role::TenantUser, Some(7), StatusCode::FORBIDDEN),
            (Role::Customer, None, StatusCode::NOT_FOUND),
        ];

        for (role, tenant_id, expected_status) in cases {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_exec_results([MockExecResult {
                    last_insert_id: 0,
                    rows_affected: 1,
                }])
                .into_connection();
            let app_state = state(db);
            let app = Router::new()
                .nest(
                    "/tenant",
                    crate::routes::tenant_routes::tenant_routes(app_state.clone()),
                )
                .with_state(app_state);
            let authorization = format!(
                "Bearer {}",
                token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
            );
            let response = app
                .oneshot(json_request(
                    "PUT",
                    "/tenant/7/listing",
                    &authorization,
                    r#"{"listed":false}"#,
                ))
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                expected_status,
                "PUT tenant listing as {role:?} for tenant {tenant_id:?}"
            );
        }
    }

    #[tokio::test]
    async fn customer_self_routes_are_restricted_to_customer_role() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, StatusCode::FORBIDDEN, StatusCode::FORBIDDEN),
            (Role::TenantOwner, Some(7), StatusCode::FORBIDDEN, StatusCode::FORBIDDEN),
            (Role::TenantUser, Some(7), StatusCode::FORBIDDEN, StatusCode::FORBIDDEN),
            (Role::Customer, None, StatusCode::OK, StatusCode::CONFLICT),
        ];

        for (role, tenant_id, expected_get, expected_post) in cases {
            for (method, path, expected_status) in [
                ("GET", "/customers/me", expected_get),
                ("POST", "/customers/me/tax-id", expected_post),
            ] {
                let mut customer = customer_model(19, 42);
                if method == "POST" {
                    customer.cpf = Some("52998224725".into());
                }
                let db = MockDatabase::new(DatabaseBackend::Postgres)
                    .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                    .append_query_results([vec![customer]])
                    .append_query_results([Vec::<entity::customer_address_entity::Model>::new()])
                    .into_connection();
                let app_state = state(db);
                let app = crate::routes::customer_routes::customer_routes(app_state.clone())
                    .with_state(app_state);
                let authorization = format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                );
                let request = if method == "GET" {
                    request(method, path, Some(&authorization))
                } else {
                    json_request(
                        method,
                        path,
                        &authorization,
                        r#"{"cpf":"11144477735"}"#,
                    )
                };
                let response = app.oneshot(request).await.unwrap();

                assert_eq!(
                    response.status(),
                    expected_status,
                    "{method} {path} as {role:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn customer_address_id_route_enforces_tenant_scope() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, StatusCode::OK),
            (Role::TenantOwner, Some(7), StatusCode::OK),
            (Role::TenantOwner, Some(8), StatusCode::NOT_FOUND),
            (Role::TenantUser, Some(7), StatusCode::OK),
            (Role::Customer, None, StatusCode::NOT_FOUND),
        ];

        for (role, tenant_id, expected_status) in cases {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![customer_address_model(90, Some(7), 19)]])
                .into_connection();
            let app_state = state(db);
            let app = crate::routes::customer_routes::customer_routes(app_state.clone())
                .with_state(app_state);
            let response = app
                .oneshot(request(
                    "GET",
                    "/customer-addresses/90",
                    Some(&format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    )),
                ))
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                expected_status,
                "GET customer address as {role:?} for tenant {tenant_id:?}"
            );
        }
    }

    #[tokio::test]
    async fn user_by_id_route_characterizes_cross_tenant_role_access() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, true, StatusCode::OK, "unrestricted"),
            (Role::TenantOwner, Some(7), false, StatusCode::NOT_FOUND, "tenant"),
            (Role::TenantOwner, Some(8), true, StatusCode::OK, "tenant"),
            (Role::TenantUser, Some(7), false, StatusCode::NOT_FOUND, "tenant"),
            (Role::Customer, None, false, StatusCode::NOT_FOUND, "deny"),
        ];

        for (role, tenant_id, target_is_in_scope, expected_status, expected_scope) in cases {
            let mut target_user = user_model_for(Role::TenantOwner, Some(8));
            target_user.id = 99;
            target_user.email = "tenant-8-user@example.test".into();
            let query_rows = if target_is_in_scope {
                vec![target_user]
            } else {
                Vec::new()
            };
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([query_rows])
                .into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = Router::new()
                .nest("/user", crate::routes::user_routes::user_routes(app_state.clone()))
                .with_state(app_state);
            let response = app
                .oneshot(request(
                    "GET",
                    "/user/99",
                    Some(&format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    )),
                ))
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                expected_status,
                "GET T2 user by ID as {role:?} for tenant {tenant_id:?}"
            );
            let transaction_log = db_for_log.into_transaction_log();
            let user_query = transaction_log
                .iter()
                .flat_map(|transaction| transaction.statements())
                .filter(|statement| statement.sql.contains("FROM \"user\""))
                .last()
                .expect("user-by-ID query should be executed");
            let where_clause = user_query
                .sql
                .split_once(" WHERE ")
                .map(|(_, clause)| clause)
                .unwrap_or_default();

            match expected_scope {
                "unrestricted" => assert!(
                    where_clause.contains("id") && !where_clause.contains("tenant_id"),
                    "SysAdmin query should select by ID without tenant filtering: {}",
                    user_query.sql
                ),
                "tenant" => assert!(
                    where_clause.contains("tenant_id") && where_clause.contains("id"),
                    "tenant role query should filter by tenant and ID: {}",
                    user_query.sql
                ),
                "deny" => assert!(
                    where_clause.contains("1 = 0") && where_clause.contains("id"),
                    "tenantless Customer query should be deny-all by ID: {}",
                    user_query.sql
                ),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }

    #[tokio::test]
    async fn user_list_route_characterizes_role_and_tenant_scope() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, 2, "unrestricted"),
            (Role::TenantOwner, Some(7), 1, "tenant"),
            (Role::TenantUser, Some(7), 0, "no_query"),
            (Role::Customer, None, 0, "no_query"),
        ];

        for (role, tenant_id, expected_count, expected_scope) in cases {
            let list_rows = match role {
                Role::SysAdmin => vec![
                    user_model_for(Role::TenantOwner, Some(7)),
                    user_model_for(Role::TenantOwner, Some(8)),
                ],
                Role::TenantOwner => vec![user_model_for(Role::TenantUser, Some(7))],
                _ => Vec::new(),
            };
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([list_rows])
                .into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = Router::new()
                .nest("/user", crate::routes::user_routes::user_routes(app_state.clone()))
                .with_state(app_state);
            let response = app
                .oneshot(request(
                    "GET",
                    "/user",
                    Some(&format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    )),
                ))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK, "GET user list as {role:?}");
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let users: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(users.as_array().unwrap().len(), expected_count, "role {role:?}");

            let transaction_log = db_for_log.into_transaction_log();
            let user_queries: Vec<_> = transaction_log
                .iter()
                .flat_map(|transaction| transaction.statements())
                .filter(|statement| statement.sql.contains("FROM \"user\""))
                .collect();
            match expected_scope {
                "unrestricted" => {
                    let list_query = user_queries.last().expect("SysAdmin list query");
                    let where_clause = list_query
                        .sql
                        .split_once(" WHERE ")
                        .map(|(_, clause)| clause)
                        .unwrap_or_default();
                    assert!(!where_clause.contains("tenant_id"), "{}", list_query.sql);
                }
                "tenant" => {
                    let list_query = user_queries.last().expect("TenantOwner list query");
                    assert!(list_query.sql.contains("tenant_id"), "{}", list_query.sql);
                }
                "no_query" => assert_eq!(user_queries.len(), 1, "only auth lookup for {role:?}"),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }

    #[tokio::test]
    async fn user_me_route_returns_authenticated_identity_for_all_roles() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None),
            (Role::TenantOwner, Some(7)),
            (Role::TenantUser, Some(7)),
            (Role::Customer, None),
        ];

        for (role, tenant_id) in cases {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([Vec::<entity::customer_entity::Model>::new()])
                .into_connection();
            let app_state = state(db);
            let app = Router::new()
                .nest("/user", crate::routes::user_routes::user_routes(app_state.clone()))
                .with_state(app_state);
            let response = app
                .oneshot(request(
                    "GET",
                    "/user/me",
                    Some(&format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    )),
                ))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK, "GET user/me as {role:?}");
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(profile["user"]["role"], role.to_string());
            assert!(profile["customer"].is_null());
            assert_eq!(profile["addresses"].as_array().unwrap().len(), 0);
        }
    }

    #[tokio::test]
    async fn user_paged_route_characterizes_role_and_tenant_scope() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, "unrestricted"),
            (Role::TenantOwner, Some(7), "tenant"),
            (Role::TenantUser, Some(7), "no_query"),
            (Role::Customer, None, "no_query"),
        ];

        for (role, tenant_id, expected_scope) in cases {
            let count_row = BTreeMap::from([(
                "num_items".to_string(),
                Value::BigInt(Some(1)),
            )]);
            let list_rows = match role {
                Role::SysAdmin => vec![user_model_for(Role::TenantOwner, Some(7))],
                Role::TenantOwner => vec![user_model_for(Role::TenantUser, Some(7))],
                _ => Vec::new(),
            };
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![count_row]])
                .append_query_results([list_rows])
                .into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = Router::new()
                .nest("/user", crate::routes::user_routes::user_routes(app_state.clone()))
                .with_state(app_state);
            let response = app
                .oneshot(request(
                    "GET",
                    "/user/paged",
                    Some(&format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    )),
                ))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK, "GET paged users as {role:?}");
            let transaction_log = db_for_log.into_transaction_log();
            let user_queries: Vec<_> = transaction_log
                .iter()
                .flat_map(|transaction| transaction.statements())
                .filter(|statement| statement.sql.contains("FROM \"user\""))
                .collect();
            match expected_scope {
                "unrestricted" => {
                    let list_query = user_queries.last().expect("SysAdmin paged query");
                    let where_clause = list_query
                        .sql
                        .split_once(" WHERE ")
                        .map(|(_, clause)| clause)
                        .unwrap_or_default();
                    assert!(!where_clause.contains("tenant_id"), "{}", list_query.sql);
                }
                "tenant" => {
                    let list_query = user_queries.last().expect("TenantOwner paged query");
                    assert!(list_query.sql.contains("tenant_id"), "{}", list_query.sql);
                }
                "no_query" => assert_eq!(user_queries.len(), 1, "only auth lookup for {role:?}"),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }

    #[tokio::test]
    async fn user_add_route_characterizes_role_and_tenant_matrix() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, 7, StatusCode::CREATED),
            (Role::TenantOwner, Some(7), 7, StatusCode::CREATED),
            (Role::TenantOwner, None, 7, StatusCode::FORBIDDEN),
            (Role::TenantUser, Some(7), 7, StatusCode::FORBIDDEN),
            (Role::Customer, None, 7, StatusCode::FORBIDDEN),
        ];
        let payload = r#"{"name":"Created user","email":"created@example.test","password":"C001-test-password","enabled":true,"firstLogin":false,"role":"TenantOwner","tenantId":7}"#;

        for (role, tenant_id, requested_tenant_id, expected_status) in cases {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![user_model_for(Role::TenantOwner, Some(requested_tenant_id))]])
                .append_query_results([vec![person_model(101, 100, Some(requested_tenant_id))]])
                .into_connection();
            let app_state = state(db);
            let app = Router::new()
                .nest("/user", crate::routes::user_routes::user_routes(app_state.clone()))
                .with_state(app_state);
            let response = app
                .oneshot(json_request(
                    "POST",
                    "/user",
                    &format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    ),
                    payload,
                ))
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                expected_status,
                "POST user as {role:?} with tenant {tenant_id:?}"
            );
        }
    }

    #[tokio::test]
    async fn user_update_route_characterizes_self_admin_and_tenant_owner_access() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, StatusCode::OK),
            (Role::TenantOwner, Some(7), StatusCode::OK),
            (Role::TenantOwner, Some(8), StatusCode::FORBIDDEN),
            (Role::TenantUser, Some(7), StatusCode::FORBIDDEN),
            (Role::Customer, None, StatusCode::FORBIDDEN),
        ];

        for (role, tenant_id, expected_status) in cases {
            let mut target_user = user_model_for(Role::TenantUser, Some(7));
            target_user.id = 99;
            target_user.email = "tenant-7-user@example.test".into();
            let mut updated_user = target_user.clone();
            updated_user.name = Some("Updated user".into());
            let requested_role = if role == Role::SysAdmin {
                "TenantOwner"
            } else {
                "TenantUser"
            };
            updated_user.role = requested_role.into();
            let mut query_batches = vec![vec![target_user.clone()]];
            if role == Role::TenantOwner && tenant_id == Some(7) {
                query_batches.push(vec![target_user.clone()]);
            }
            if matches!(role, Role::SysAdmin | Role::TenantOwner)
                && !(role == Role::TenantOwner && tenant_id == Some(8))
            {
                query_batches.push(vec![updated_user]);
            }

            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results(query_batches)
                .into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = Router::new()
                .nest("/user", crate::routes::user_routes::user_routes(app_state.clone()))
                .with_state(app_state);
            let update_body = format!(
                r#"{{"name":"Updated user","email":"tenant-7-user@example.test","enabled":true,"role":"{requested_role}","tenantId":7}}"#
            );
            let authorization = format!(
                "Bearer {}",
                token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
            );
            let response = app
                .oneshot(json_request(
                    "PUT",
                    "/user/99",
                    &authorization,
                    &update_body,
                ))
                .await
                .unwrap();

            let actual_status = response.status();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let transaction_log = db_for_log.into_transaction_log();
            let executed_sql: Vec<_> = transaction_log
                .iter()
                .flat_map(|transaction| transaction.statements())
                .map(|statement| statement.sql.clone())
                .collect();
            assert_eq!(
                actual_status,
                expected_status,
                "PUT user 99 as {role:?} for tenant {tenant_id:?}; response: {}; SQL: {executed_sql:?}",
                String::from_utf8_lossy(&body)
            );
        }
    }

    #[tokio::test]
    async fn user_update_route_refuses_other_users_for_non_admin_roles() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::TenantOwner, Some(7), true),
            (Role::TenantUser, Some(7), false),
            (Role::Customer, None, false),
        ];

        for (role, tenant_id, mock_target_read) in cases {
            let mut query_results = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
            if mock_target_read {
                let mut target_user = user_model_for(Role::TenantOwner, Some(8));
                target_user.id = 99;
                target_user.email = "tenant-8-user@example.test".into();
                query_results = query_results.append_query_results([vec![target_user]]);
            }
            let db = query_results.into_connection();
            let app_state = state(db);
            let app = Router::new()
                .nest("/user", crate::routes::user_routes::user_routes(app_state.clone()))
                .with_state(app_state);
            let authorization = format!(
                "Bearer {}",
                token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
            );
            let response = app
                .oneshot(json_request(
                    "PUT",
                    "/user/99",
                    &authorization,
                    r#"{"email":"target@example.test","enabled":true,"role":"TenantUser"}"#,
                ))
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                StatusCode::FORBIDDEN,
                "PUT another user's record as {role:?} for tenant {tenant_id:?}"
            );
        }
    }

    #[tokio::test]
    async fn user_change_password_route_scopes_user_lookup_by_role() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }

        let cases = [
            (Role::SysAdmin, None, "unrestricted"),
            (Role::TenantOwner, Some(7), "tenant"),
            (Role::TenantUser, Some(7), "tenant"),
            (Role::Customer, None, "deny"),
        ];

        for (role, tenant_id, expected_scope) in cases {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = Router::new()
                .nest("/user", crate::routes::user_routes::user_routes(app_state.clone()))
                .with_state(app_state);
            let authorization = format!(
                "Bearer {}",
                token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
            );
            let response = app
                .oneshot(json_request(
                    "PUT",
                    "/user/change-password",
                    &authorization,
                    r#"{"currentPassword":"wrong-test-password","newPassword":"new-test-password"}"#,
                ))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::BAD_REQUEST, "role {role:?}");
            let user_query = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .filter(|statement| statement.sql.contains("FROM \"user\""))
                .last()
                .expect("change-password user lookup should execute");
            let where_clause = user_query
                .sql
                .split_once(" WHERE ")
                .map(|(_, clause)| clause)
                .unwrap_or_default();

            match expected_scope {
                "unrestricted" => assert!(!where_clause.contains("tenant_id"), "{}", user_query.sql),
                "tenant" => assert!(where_clause.contains("tenant_id"), "{}", user_query.sql),
                "deny" => assert!(where_clause.contains("1 = 0"), "{}", user_query.sql),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }
}
