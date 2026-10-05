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
mod c001_auth_middleware_tests {
    use super::*;
    use axum::{
        Extension, Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
    };
    use business::{
        domain::{enums::Role, shipping::ShippingOption, user::User},
        gateway::shipping_provider_gateway::{
            ShippingProviderError, ShippingProviderGateway, ShippingRequest,
        },
        sea_orm::{DatabaseBackend, DbConn, MockDatabase},
    };
    use chrono::Utc;
    use entity::user_entity;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use crate::authentication::authentication_middleware::authentication;
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
        let now = Utc::now().naive_utc();
        user_entity::Model {
            id: 42,
            uuid: uuid::Uuid::new_v4(),
            name: Some("C001 Customer".into()),
            email: "c001@example.test".into(),
            password: String::new(),
            first_login: false,
            enabled: true,
            tenant_id: None,
            role: "Customer".into(),
            blocked_reason: None,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        }
    }

    fn token(expires_at: i64) -> String {
        let claims = business::domain::access_token::Claims {
            sub: "c001@example.test".into(),
            exp: expires_at,
            uuid: "c001-user".into(),
            name: "C001 Customer".into(),
            user_id: 42,
            role: Role::Customer,
            tenant_id: None,
        };
        encode(
            &Header::new(Algorithm::HS512),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .unwrap()
    }

    fn request(authorization: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().uri("/protected");
        if let Some(value) = authorization {
            builder = builder.header("authorization", value);
        }
        builder.body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn authentication_middleware_characterizes_bearer_status_and_expiry_gap() {
        unsafe { std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET); }
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model()], vec![user_model()]])
            .into_connection();
        let app = Router::new()
            .route(
                "/protected",
                get(|Extension(user): Extension<User>| async move { user.role.to_string() }),
            )
            .route_layer(middleware::from_fn_with_state(state(db), authentication));

        assert_eq!(
            app.clone().oneshot(request(None)).await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            app.clone()
                .oneshot(request(Some("Basic not-a-bearer")))
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            app.clone()
                .oneshot(request(Some("Bearer malformed-token")))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            app.clone()
                .oneshot(request(Some(&format!("Bearer {}", token(Utc::now().timestamp() - 60)))))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            app.oneshot(request(Some(&format!("Bearer {}", token(Utc::now().timestamp() + 3600)))))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
    }
}
