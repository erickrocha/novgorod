use crate::authentication::authentication_middleware::authentication;
use crate::{AppState, infrastructure};
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

fn person_address_model(
    id: i64,
    person_id: i64,
    tenant_id: Option<i64>,
) -> entity::person_address_entity::Model {
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

async fn assert_routes_require_auth(app: &Router, routes: &[(&str, &str)]) -> usize {
    for (method, path) in routes {
        let response = app
            .clone()
            .oneshot(request(method, path, None))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "unauthenticated {method} {path} should be blocked by the shared middleware"
        );
    }
    routes.len()
}

#[tokio::test]
async fn cart_read_routes_characterize_role_tenant_and_customer_owner() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    fn cart_row(tenant_id: Option<i64>, customer_id: Option<i64>) -> entity::cart_entity::Model {
        let now = Utc::now().naive_utc();
        entity::cart_entity::Model {
            id: 99,
            uuid: uuid::Uuid::new_v4(),
            tenant_id,
            customer_id,
            status: "active".into(),
            expires_at: None,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        }
    }
    fn item_row(tenant_id: Option<i64>) -> entity::cart_item_entity::Model {
        let now = Utc::now().naive_utc();
        entity::cart_item_entity::Model {
            id: 199,
            uuid: uuid::Uuid::new_v4(),
            tenant_id,
            cart_id: 99,
            sku_id: 55,
            quantity: 2,
            unit_price_cents: 5000,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        }
    }

    let cases = [
        (Role::SysAdmin, None, "unrestricted", None),
        (Role::TenantOwner, Some(8), "tenant", None),
        (Role::TenantOwner, Some(7), "foreign", None),
        (Role::TenantUser, Some(7), "foreign", None),
        (Role::Customer, None, "customer", Some(77)),
    ];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(1)))]);
    let routes = [
        ("/carts", "cart", false, false),
        ("/carts/paged", "cart", true, false),
        ("/carts/99", "cart", false, true),
        ("/cart-items", "cart_item", false, false),
        ("/cart-items/paged", "cart_item", true, false),
        ("/cart-items/199", "cart_item", false, true),
    ];

    for (role, tenant_id, expected_scope, customer_id) in cases {
        for (path, table, paged, by_id) in routes {
            let tenant = customer_id.map_or(Some(8), |_| None);
            let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
            if role == Role::Customer {
                db = db.append_query_results([vec![customer_model(77, 42)]]);
            }
            if paged {
                db = db.append_query_results([vec![count_row.clone()]]);
            }
            db = match table {
                "cart" => db.append_query_results([vec![cart_row(tenant, customer_id)]]),
                "cart_item" => db.append_query_results([vec![item_row(tenant)]]),
                _ => unreachable!("unknown cart table"),
            };
            let db = db.into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = crate::routes::cart_routes::cart_routes(app_state.clone())
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    crate::commons::tenant_context::tenant_context,
                ))
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
            let status = response.status();
            let response_body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();

            let expected_status = if expected_scope == "foreign" && by_id {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::OK
            };
            assert_eq!(
                status,
                expected_status,
                "GET {path} as {role:?} in scope {expected_scope}: {}",
                String::from_utf8_lossy(&response_body)
            );

            let queries: Vec<_> = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .filter(|statement| statement.sql.contains(&format!("FROM \"{table}\"")))
                .collect();
            let query = if paged {
                assert_eq!(queries.len(), 2, "{path} should count and fetch rows");
                &queries[1]
            } else {
                assert_eq!(queries.len(), 1, "{path} should execute one table query");
                &queries[0]
            };
            let where_clause = query
                .sql
                .split_once(" WHERE ")
                .map(|(_, clause)| clause)
                .unwrap_or_default();
            match expected_scope {
                "unrestricted" => assert!(!where_clause.contains("tenant_id"), "{}", query.sql),
                "tenant" | "foreign" => {
                    assert!(where_clause.contains("tenant_id"), "{}", query.sql)
                }
                "customer" => {
                    assert!(where_clause.contains("customer_id"), "{}", query.sql);
                    if table == "cart_item" {
                        assert!(
                            query.sql.contains("SELECT") && query.sql.contains("customer_id"),
                            "{}",
                            query.sql
                        );
                    }
                }
                _ => unreachable!("unknown expected scope"),
            }
        }
    }
}

#[tokio::test]
async fn cart_update_enforces_tenant_and_customer_ownership() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, Some(8), None, StatusCode::OK),
        (Role::TenantOwner, Some(7), Some(7), None, StatusCode::OK),
        (
            Role::TenantOwner,
            Some(7),
            Some(8),
            None,
            StatusCode::FORBIDDEN,
        ),
        (
            Role::TenantUser,
            Some(7),
            Some(8),
            None,
            StatusCode::FORBIDDEN,
        ),
        (Role::Customer, None, None, Some(77), StatusCode::OK),
    ];
    let now = Utc::now().naive_utc();
    let payload = r#"{"tenantId":8,"customerId":999,"status":"abandoned"}"#;

    for (role, token_tenant, row_tenant, customer_id, expected_status) in cases {
        let cart = entity::cart_entity::Model {
            id: 99,
            uuid: uuid::Uuid::new_v4(),
            tenant_id: row_tenant,
            customer_id,
            status: "active".into(),
            expires_at: None,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        };
        let updated = entity::cart_entity::Model {
            status: "abandoned".into(),
            ..cart.clone()
        };
        let mut db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), token_tenant)]]);
        if let Some(customer) = customer_id {
            db = db.append_query_results([vec![customer_model(customer, 42)]]);
        }
        let db = db
            .append_query_results([vec![cart]])
            .append_query_results([vec![updated]])
            .into_connection();
        let app_state = state(db);
        let app = crate::routes::cart_routes::cart_routes(app_state.clone())
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                crate::commons::tenant_context::tenant_context,
            ))
            .with_state(app_state);
        let response = app
            .oneshot(json_request(
                "PUT",
                "/carts/99",
                &format!(
                    "Bearer {}",
                    token_for(role.clone(), token_tenant, Utc::now().timestamp() + 3600)
                ),
                payload,
            ))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            expected_status,
            "PUT cart as {role:?} for row tenant {row_tenant:?}"
        );
        if expected_status == StatusCode::OK {
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(
                body["customerId"],
                customer_id.map_or(serde_json::Value::Null, serde_json::Value::from)
            );
        }
    }
}

#[tokio::test]
async fn cart_create_uses_authenticated_tenant_and_customer() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, Some(8), Some(999), Some(8), Some(999)),
        (
            Role::TenantOwner,
            Some(7),
            Some(8),
            Some(999),
            Some(7),
            Some(999),
        ),
        (
            Role::TenantUser,
            Some(7),
            Some(8),
            Some(999),
            Some(7),
            Some(999),
        ),
        (Role::Customer, None, Some(8), Some(999), None, Some(77)),
    ];

    for (
        role,
        token_tenant,
        requested_tenant,
        requested_customer,
        expected_tenant,
        expected_customer,
    ) in cases
    {
        let now = Utc::now().naive_utc();
        let cart = entity::cart_entity::Model {
            id: 99,
            uuid: uuid::Uuid::new_v4(),
            tenant_id: expected_tenant,
            customer_id: expected_customer,
            status: "active".into(),
            expires_at: None,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        };
        let mut db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), token_tenant)]]);
        if role == Role::Customer {
            db = db.append_query_results([vec![customer_model(77, 42)]]);
        }
        let db = db.append_query_results([vec![cart]]).into_connection();
        let app_state = state(db);
        let app = crate::routes::cart_routes::cart_routes(app_state.clone())
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                crate::commons::tenant_context::tenant_context,
            ))
            .with_state(app_state);
        let response = app
            .oneshot(json_request(
                "POST",
                "/carts",
                &format!(
                    "Bearer {}",
                    token_for(role.clone(), token_tenant, Utc::now().timestamp() + 3600)
                ),
                &format!(
                    "{{\"tenantId\":{},\"customerId\":{},\"status\":\"active\"}}",
                    requested_tenant.unwrap(),
                    requested_customer.unwrap()
                ),
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "POST /carts as {role:?}"
        );
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let cart: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            cart["tenantId"],
            expected_tenant.map_or(serde_json::Value::Null, serde_json::Value::from)
        );
        assert_eq!(
            cart["customerId"],
            expected_customer.map_or(serde_json::Value::Null, serde_json::Value::from)
        );
    }
}

#[tokio::test]
async fn cart_item_create_and_update_keep_authenticated_scope() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, Some(8), None),
        (Role::TenantOwner, Some(7), Some(7), None),
        (Role::TenantUser, Some(7), Some(7), None),
        (Role::Customer, None, None, Some(77)),
    ];
    let now = Utc::now().naive_utc();
    let input = r#"{"tenantId":8,"cartId":99,"skuId":55,"quantity":3,"unitPriceCents":6000}"#;

    for (role, token_tenant, expected_tenant, customer_id) in cases {
        let saved_item = entity::cart_item_entity::Model {
            id: 199,
            uuid: uuid::Uuid::new_v4(),
            tenant_id: expected_tenant,
            cart_id: 99,
            sku_id: 55,
            quantity: 3,
            unit_price_cents: 6000,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        };
        let mut create_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), token_tenant)]]);
        if let Some(customer) = customer_id {
            let cart = entity::cart_entity::Model {
                id: 99,
                uuid: uuid::Uuid::new_v4(),
                tenant_id: None,
                customer_id: Some(customer),
                status: "active".into(),
                expires_at: None,
                created_at: now,
                created_by: None,
                updated_at: now,
                updated_by: None,
            };
            create_db = create_db
                .append_query_results([vec![customer_model(customer, 42)]])
                .append_query_results([vec![cart]]);
        }
        let create_db = create_db
            .append_query_results([vec![saved_item.clone()]])
            .into_connection();
        let create_state = state(create_db);
        let create_app = crate::routes::cart_routes::cart_routes(create_state.clone())
            .layer(middleware::from_fn_with_state(
                create_state.clone(),
                crate::commons::tenant_context::tenant_context,
            ))
            .with_state(create_state);
        let create_response = create_app
            .oneshot(json_request(
                "POST",
                "/cart-items",
                &format!(
                    "Bearer {}",
                    token_for(role.clone(), token_tenant, Utc::now().timestamp() + 3600)
                ),
                input,
            ))
            .await
            .unwrap();
        assert_eq!(
            create_response.status(),
            StatusCode::CREATED,
            "POST as {role:?}"
        );
        let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let created: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
        assert_eq!(
            created["tenantId"],
            expected_tenant.map_or(serde_json::Value::Null, serde_json::Value::from)
        );

        let mut update_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), token_tenant)]]);
        if let Some(customer) = customer_id {
            update_db = update_db.append_query_results([vec![customer_model(customer, 42)]]);
        }
        update_db = update_db
            .append_query_results([vec![saved_item.clone()]])
            .append_query_results([vec![saved_item]]);
        let update_state = state(update_db.into_connection());
        let update_app = crate::routes::cart_routes::cart_routes(update_state.clone())
            .layer(middleware::from_fn_with_state(
                update_state.clone(),
                crate::commons::tenant_context::tenant_context,
            ))
            .with_state(update_state);
        let update_response = update_app
            .oneshot(json_request(
                "PUT",
                "/cart-items/199",
                &format!(
                    "Bearer {}",
                    token_for(role.clone(), token_tenant, Utc::now().timestamp() + 3600)
                ),
                input,
            ))
            .await
            .unwrap();
        assert_eq!(update_response.status(), StatusCode::OK, "PUT as {role:?}");
        let update_body = axum::body::to_bytes(update_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let updated: serde_json::Value = serde_json::from_slice(&update_body).unwrap();
        assert_eq!(
            updated["tenantId"],
            expected_tenant.map_or(serde_json::Value::Null, serde_json::Value::from)
        );
        assert_eq!(updated["cartId"], 99);
    }
}

#[tokio::test]
async fn marketing_create_routes_use_authenticated_tenant() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, Some(8), Some(8), StatusCode::CREATED),
        (
            Role::TenantOwner,
            Some(7),
            Some(8),
            Some(7),
            StatusCode::CREATED,
        ),
        (
            Role::TenantUser,
            Some(7),
            Some(8),
            Some(7),
            StatusCode::CREATED,
        ),
        (Role::Customer, None, Some(8), None, StatusCode::FORBIDDEN),
    ];
    let now = Utc::now().naive_utc();
    let routes = [
        (
            "/campaigns",
            "campaign",
            r#"{"tenantId":8,"name":" Spring Sale ","campaignType":"discount","value":10,"scope":"all","startsAt":"2026-10-01T00:00:00","endsAt":"2026-10-31T00:00:00","active":true}"#,
        ),
        (
            "/campaign-targets",
            "campaign_target",
            r#"{"tenantId":8,"campaignId":10,"targetType":"product","targetId":20}"#,
        ),
        (
            "/coupons",
            "coupon",
            r#"{"tenantId":8,"code":"SAVE10","couponType":"PERCENTAGE","value":10}"#,
        ),
    ];

    for (role, user_tenant, requested_tenant, expected_tenant, expected_status) in cases {
        for (path, table, payload) in routes {
            let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), user_tenant)]]);
            if let Some(tenant_id) = expected_tenant {
                db = match table {
                    "campaign" => db.append_query_results([vec![entity::campaign_entity::Model {
                        id: 99,
                        uuid: uuid::Uuid::new_v4(),
                        tenant_id: Some(tenant_id),
                        name: "Spring Sale".into(),
                        campaign_type: "discount".into(),
                        value: 10,
                        scope: "all".into(),
                        starts_at: now,
                        ends_at: now,
                        active: true,
                        created_at: now,
                        created_by: None,
                        updated_at: now,
                        updated_by: None,
                    }]]),
                    "campaign_target" => {
                        db.append_query_results([vec![entity::campaign_target_entity::Model {
                            id: 99,
                            uuid: uuid::Uuid::new_v4(),
                            tenant_id: Some(tenant_id),
                            campaign_id: 10,
                            target_type: "product".into(),
                            target_id: 20,
                            created_at: now,
                            created_by: None,
                        }]])
                    }
                    "coupon" => db.append_query_results([vec![entity::coupon_entity::Model {
                        id: 99,
                        uuid: uuid::Uuid::new_v4(),
                        tenant_id: Some(tenant_id),
                        code: "SAVE10".into(),
                        campaign_id: None,
                        coupon_type: "PERCENTAGE".into(),
                        value: 10,
                        min_order_cents: None,
                        max_uses: None,
                        max_uses_per_customer: None,
                        starts_at: None,
                        expires_at: None,
                        active: true,
                        created_at: now,
                        created_by: None,
                        updated_at: now,
                        updated_by: None,
                    }]]),
                    _ => unreachable!("unknown marketing table"),
                };
            }
            let app_state = state(db.into_connection());
            let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
                .with_state(app_state);
            let response = app
                .oneshot(json_request(
                    "POST",
                    path,
                    &format!(
                        "Bearer {}",
                        token_for(role.clone(), user_tenant, Utc::now().timestamp() + 3600)
                    ),
                    payload,
                ))
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                expected_status,
                "POST {path} as {role:?} with requested tenant {requested_tenant:?}"
            );
            if expected_status == StatusCode::CREATED {
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let saved: serde_json::Value = serde_json::from_slice(&body).unwrap();
                assert_eq!(saved["tenantId"], expected_tenant.unwrap());
            }
        }
    }
}

#[tokio::test]
async fn marketing_updates_refuse_foreign_tenant_rows() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];
    let now = Utc::now().naive_utc();
    let updates = [
        (
            "/campaigns/99",
            "campaign",
            r#"{"name":"Updated","campaignType":"discount","value":10,"scope":"all","startsAt":"2026-10-01T00:00:00","endsAt":"2026-10-31T00:00:00"}"#,
        ),
        (
            "/campaign-targets/99",
            "campaign_target",
            r#"{"campaignId":10,"targetType":"product","targetId":20}"#,
        ),
        (
            "/coupons/99",
            "coupon",
            r#"{"code":"SAVE10","couponType":"PERCENTAGE","value":10}"#,
        ),
    ];

    for (role, tenant_id) in roles {
        for (path, table, payload) in updates {
            let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
            db = match table {
                "campaign" => db.append_query_results([vec![entity::campaign_entity::Model {
                    id: 99,
                    uuid: uuid::Uuid::new_v4(),
                    tenant_id: Some(8),
                    name: "Existing".into(),
                    campaign_type: "discount".into(),
                    value: 5,
                    scope: "all".into(),
                    starts_at: now,
                    ends_at: now,
                    active: true,
                    created_at: now,
                    created_by: None,
                    updated_at: now,
                    updated_by: None,
                }]]),
                "campaign_target" => {
                    db.append_query_results([vec![entity::campaign_target_entity::Model {
                        id: 99,
                        uuid: uuid::Uuid::new_v4(),
                        tenant_id: Some(8),
                        campaign_id: 10,
                        target_type: "product".into(),
                        target_id: 20,
                        created_at: now,
                        created_by: None,
                    }]])
                }
                "coupon" => db.append_query_results([vec![entity::coupon_entity::Model {
                    id: 99,
                    uuid: uuid::Uuid::new_v4(),
                    tenant_id: Some(8),
                    code: "OLD".into(),
                    campaign_id: None,
                    coupon_type: "PERCENTAGE".into(),
                    value: 5,
                    min_order_cents: None,
                    max_uses: None,
                    max_uses_per_customer: None,
                    starts_at: None,
                    expires_at: None,
                    active: true,
                    created_at: now,
                    created_by: None,
                    updated_at: now,
                    updated_by: None,
                }]]),
                _ => unreachable!("unknown marketing table"),
            };
            let app_state = state(db.into_connection());
            let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
                .with_state(app_state);
            let response = app
                .oneshot(json_request(
                    "PUT",
                    path,
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
                StatusCode::FORBIDDEN,
                "PUT {path} as {role:?} should refuse tenant-8 row"
            );
        }
    }
}

#[tokio::test]
async fn payment_config_is_customer_only() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None),
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
    ];

    for (role, tenant_id) in roles {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .into_connection();
        let app_state = state(db);
        let app =
            crate::routes::order_routes::order_routes(app_state.clone()).with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/checkout/payment-config",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "payment config should refuse {role:?} without provider access"
        );
    }
}

#[tokio::test]
async fn coupon_redemption_creation_is_sysadmin_only() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, None, StatusCode::FORBIDDEN),
        (Role::SysAdmin, None, Some(8), StatusCode::BAD_REQUEST),
        (Role::TenantOwner, Some(7), Some(8), StatusCode::FORBIDDEN),
        (Role::TenantUser, Some(7), Some(8), StatusCode::FORBIDDEN),
        (Role::Customer, None, Some(8), StatusCode::FORBIDDEN),
    ];

    for (role, tenant_id, requested_tenant, expected_status) in cases {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .into_connection();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let payload = format!(
            "{{\"tenantId\":{},\"couponId\":1,\"orderId\":2,\"customerId\":3}}",
            requested_tenant.map_or("null".to_string(), |id| id.to_string())
        );
        let response = app
            .oneshot(json_request(
                "POST",
                "/coupon-redemptions",
                &format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                ),
                &payload,
            ))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            expected_status,
            "POST coupon redemption as {role:?} with tenant {requested_tenant:?}"
        );
    }
}

#[tokio::test]
async fn catalog_create_routes_stamp_authenticated_tenant() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None, Some(8), Some(8), StatusCode::CREATED),
        (
            Role::TenantOwner,
            Some(7),
            Some(8),
            Some(7),
            StatusCode::CREATED,
        ),
        (
            Role::TenantUser,
            Some(7),
            Some(8),
            Some(7),
            StatusCode::CREATED,
        ),
        (Role::Customer, None, Some(8), None, StatusCode::FORBIDDEN),
    ];
    let routes = [
        (
            "/categories",
            "category",
            r#"{"tenantId":8,"name":"New category","slug":"new-category","active":true}"#,
        ),
        (
            "/products",
            "product",
            r#"{"tenantId":8,"name":"New product","slug":"new-product","active":true,"ncm":"12345678","origemMercadoria":0}"#,
        ),
        (
            "/skus",
            "sku",
            r#"{"tenantId":8,"productId":44,"code":"SKU-44","variantKey":"default","priceCents":5000,"active":true}"#,
        ),
        (
            "/product-categories",
            "product_category",
            r#"{"tenantId":8,"productId":44,"categoryId":66,"isPrimary":true}"#,
        ),
        (
            "/sku-stocks",
            "sku_stock",
            r#"{"tenantId":8,"skuId":55,"quantity":5,"reserved":0}"#,
        ),
        (
            "/sku-attributes",
            "sku_attribute_value",
            r#"{"tenantId":8,"productId":44,"skuId":55,"productAttributeId":66,"attributeId":66,"attributeValueId":77}"#,
        ),
    ];
    let now = Utc::now().naive_utc();

    for (role, user_tenant, requested_tenant, expected_tenant, expected_status) in roles {
        for (path, table, payload) in routes {
            let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), user_tenant)]]);
            if expected_status == StatusCode::CREATED {
                if table == "sku" {
                    db = db.append_query_results([vec![entity::product_entity::Model {
                        id: 44,
                        uuid: uuid::Uuid::new_v4(),
                        tenant_id: expected_tenant,
                        name: "New product".into(),
                        slug: "new-product".into(),
                        description: None,
                        brand: None,
                        active: true,
                        ncm: "12345678".into(),
                        cest: None,
                        origem_mercadoria: 0,
                        created_at: now,
                        created_by: None,
                        updated_at: now,
                        updated_by: None,
                    }]]);
                }
                db = match table {
                    "category" => db.append_query_results([vec![entity::category_entity::Model {
                        id: 10,
                        uuid: uuid::Uuid::new_v4(),
                        tenant_id: expected_tenant,
                        name: "New category".into(),
                        slug: "new-category".into(),
                        parent_id: None,
                        active: true,
                        created_at: now,
                        created_by: None,
                        updated_at: now,
                        updated_by: None,
                    }]]),
                    "product" => db.append_query_results([vec![entity::product_entity::Model {
                        id: 44,
                        uuid: uuid::Uuid::new_v4(),
                        tenant_id: expected_tenant,
                        name: "New product".into(),
                        slug: "new-product".into(),
                        description: None,
                        brand: None,
                        active: true,
                        ncm: "12345678".into(),
                        cest: None,
                        origem_mercadoria: 0,
                        created_at: now,
                        created_by: None,
                        updated_at: now,
                        updated_by: None,
                    }]]),
                    "sku" => db.append_query_results([vec![entity::sku_entity::Model {
                        id: 55,
                        uuid: uuid::Uuid::new_v4(),
                        tenant_id: expected_tenant,
                        product_id: 44,
                        code: "SKU-44".into(),
                        variant_key: "default".into(),
                        price_cents: 5000,
                        compare_at_price_cents: None,
                        weight_g: None,
                        width_mm: None,
                        height_mm: None,
                        length_mm: None,
                        active: true,
                        created_at: now,
                        created_by: None,
                        updated_at: now,
                        updated_by: None,
                    }]]),
                    "product_category" => {
                        db.append_query_results([vec![entity::product_category_entity::Model {
                            id: 77,
                            uuid: uuid::Uuid::new_v4(),
                            tenant_id: expected_tenant,
                            product_id: 44,
                            category_id: 66,
                            is_primary: true,
                            created_at: now,
                            created_by: None,
                        }]])
                    }
                    "sku_stock" => {
                        db.append_query_results([vec![entity::sku_stock_entity::Model {
                            id: 88,
                            uuid: uuid::Uuid::new_v4(),
                            tenant_id: expected_tenant,
                            sku_id: 55,
                            warehouse_id: None,
                            quantity: 5,
                            reserved: 0,
                            created_at: now,
                            created_by: None,
                            updated_at: now,
                            updated_by: None,
                        }]])
                    }
                    "sku_attribute_value" => {
                        db.append_query_results([vec![entity::sku_attribute_value_entity::Model {
                            id: 99,
                            uuid: uuid::Uuid::new_v4(),
                            tenant_id: expected_tenant,
                            product_id: 44,
                            sku_id: 55,
                            product_attribute_id: 66,
                            attribute_id: 66,
                            attribute_value_id: 77,
                            created_at: now,
                            created_by: None,
                            updated_at: now,
                            updated_by: None,
                        }]])
                    }
                    _ => unreachable!("unknown catalog table"),
                };
            }
            let app_state = state(db.into_connection());
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
                .with_state(app_state);
            let response = app
                .oneshot(json_request(
                    "POST",
                    path,
                    &format!(
                        "Bearer {}",
                        token_for(role.clone(), user_tenant, Utc::now().timestamp() + 3600)
                    ),
                    payload,
                ))
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                expected_status,
                "POST {path} as {role:?} requesting tenant {requested_tenant:?}"
            );
            if expected_status == StatusCode::CREATED {
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let saved: serde_json::Value = serde_json::from_slice(&body).unwrap();
                assert_eq!(saved["tenantId"], expected_tenant.unwrap());
            }
        }
    }
}

#[tokio::test]
async fn catalog_mutations_refuse_foreign_tenant_rows() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];
    let routes = [
        (
            "PUT",
            "/categories/99",
            "category",
            r#"{"tenantId":8,"name":"Updated","slug":"updated","active":true}"#,
        ),
        (
            "PUT",
            "/products/99",
            "product",
            r#"{"tenantId":8,"name":"Updated","slug":"updated","active":true,"ncm":"12345678","origemMercadoria":0}"#,
        ),
        (
            "PUT",
            "/skus/99",
            "sku",
            r#"{"tenantId":8,"productId":44,"code":"SKU","variantKey":"default","priceCents":100,"active":true}"#,
        ),
        (
            "PUT",
            "/product-categories/99",
            "product_category",
            r#"{"tenantId":8,"productId":44,"categoryId":66,"isPrimary":true}"#,
        ),
        ("DELETE", "/product-categories/99", "product_category", ""),
        (
            "PUT",
            "/sku-stocks/99",
            "sku_stock",
            r#"{"tenantId":8,"skuId":55,"quantity":2}"#,
        ),
        ("DELETE", "/sku-stocks/99", "sku_stock", ""),
        (
            "PUT",
            "/sku-attributes/99",
            "sku_attribute_value",
            r#"{"tenantId":8,"productId":44,"skuId":55,"productAttributeId":66,"attributeId":66,"attributeValueId":77}"#,
        ),
        ("DELETE", "/sku-attributes/99", "sku_attribute_value", ""),
        (
            "PUT",
            "/sku-attribute-values/99",
            "sku_attribute_value",
            r#"{"tenantId":8,"productId":44,"skuId":55,"productAttributeId":66,"attributeId":66,"attributeValueId":77}"#,
        ),
        (
            "DELETE",
            "/sku-attribute-values/99",
            "sku_attribute_value",
            "",
        ),
    ];
    let now = Utc::now().naive_utc();

    for (role, tenant_id) in roles {
        for (method, path, table, body) in routes {
            let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
            db = match table {
                "category" => db.append_query_results([vec![entity::category_entity::Model {
                    id: 99,
                    uuid: uuid::Uuid::new_v4(),
                    tenant_id: Some(8),
                    name: "Old".into(),
                    slug: "old".into(),
                    parent_id: None,
                    active: true,
                    created_at: now,
                    created_by: None,
                    updated_at: now,
                    updated_by: None,
                }]]),
                "product" => db.append_query_results([vec![entity::product_entity::Model {
                    id: 99,
                    uuid: uuid::Uuid::new_v4(),
                    tenant_id: Some(8),
                    name: "Old".into(),
                    slug: "old".into(),
                    description: None,
                    brand: None,
                    active: true,
                    ncm: "12345678".into(),
                    cest: None,
                    origem_mercadoria: 0,
                    created_at: now,
                    created_by: None,
                    updated_at: now,
                    updated_by: None,
                }]]),
                "sku" => db.append_query_results([vec![entity::sku_entity::Model {
                    id: 99,
                    uuid: uuid::Uuid::new_v4(),
                    tenant_id: Some(8),
                    product_id: 44,
                    code: "SKU".into(),
                    variant_key: "default".into(),
                    price_cents: 100,
                    compare_at_price_cents: None,
                    weight_g: None,
                    width_mm: None,
                    height_mm: None,
                    length_mm: None,
                    active: true,
                    created_at: now,
                    created_by: None,
                    updated_at: now,
                    updated_by: None,
                }]]),
                "product_category" => {
                    db.append_query_results([vec![entity::product_category_entity::Model {
                        id: 99,
                        uuid: uuid::Uuid::new_v4(),
                        tenant_id: Some(8),
                        product_id: 44,
                        category_id: 66,
                        is_primary: true,
                        created_at: now,
                        created_by: None,
                    }]])
                }
                "sku_stock" => db.append_query_results([vec![entity::sku_stock_entity::Model {
                    id: 99,
                    uuid: uuid::Uuid::new_v4(),
                    tenant_id: Some(8),
                    sku_id: 55,
                    warehouse_id: None,
                    quantity: 2,
                    reserved: 0,
                    created_at: now,
                    created_by: None,
                    updated_at: now,
                    updated_by: None,
                }]]),
                "sku_attribute_value" => {
                    db.append_query_results([vec![entity::sku_attribute_value_entity::Model {
                        id: 99,
                        uuid: uuid::Uuid::new_v4(),
                        tenant_id: Some(8),
                        product_id: 44,
                        sku_id: 55,
                        product_attribute_id: 66,
                        attribute_id: 66,
                        attribute_value_id: 77,
                        created_at: now,
                        created_by: None,
                        updated_at: now,
                        updated_by: None,
                    }]])
                }
                _ => unreachable!("unknown catalog table"),
            };
            let app_state = state(db.into_connection());
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
                .with_state(app_state);
            let authorization = format!(
                "Bearer {}",
                token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
            );
            let req = if method == "DELETE" {
                request(method, path, Some(&authorization))
            } else {
                json_request(method, path, &authorization, body)
            };
            let response = app.oneshot(req).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::FORBIDDEN,
                "{method} {path} as {role:?} should refuse tenant-8 row"
            );
        }
    }
}

#[tokio::test]
async fn catalog_import_role_gate_allows_only_admin_and_owner() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, 400),
        (Role::TenantOwner, Some(7), 400),
        (Role::TenantUser, Some(7), 403),
        (Role::Customer, None, 403),
    ];

    for (role, tenant_id, expected_status) in cases {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .into_connection();
        let app_state = state(db);
        let app = crate::routes::resource_routes::resources_routes(app_state.clone())
            .with_state(app_state);
        let authorization = format!(
            "Bearer {}",
            token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
        );
        let request = Request::builder()
                .method("POST")
                .uri("/catalog/import")
                .header("authorization", authorization)
                .header("content-type", "multipart/form-data; boundary=c001")
                .body(Body::from(
                    "--c001\r\nContent-Disposition: form-data; name=\"tenantId\"\r\n\r\n8\r\n--c001\r\nContent-Disposition: form-data; name=\"editedRows\"\r\n\r\n[]\r\n--c001--\r\n",
                ))
                .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status().as_u16(),
            expected_status,
            "POST /catalog/import as {role:?}"
        );
    }
}

#[tokio::test]
async fn purchase_create_requires_idempotency_key_before_checkout() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![user_model_for(Role::Customer, None)]])
        .into_connection();
    let app_state = state(db);
    let app = crate::routes::order_routes::order_routes(app_state.clone())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            crate::commons::tenant_context::tenant_context,
        ))
        .with_state(app_state);
    let response = app
        .oneshot(json_request(
            "POST",
            "/purchases",
            &format!(
                "Bearer {}",
                token_for(Role::Customer, None, Utc::now().timestamp() + 3600)
            ),
            r#"{"quoteId":123,"email":"buyer@example.test","phone":"11999999999"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn order_and_purchase_reads_hide_foreign_resources() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];
    let routes = [
        ("/orders/99", "order"),
        ("/orders/99/status-history", "order"),
        ("/purchases/99", "purchase"),
        ("/purchases/99/payments", "purchase"),
    ];
    let now = Utc::now().naive_utc();

    for (role, tenant_id) in roles {
        for (path, resource) in routes {
            let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
            if role == Role::Customer {
                db = db.append_query_results([vec![customer_model(77, 42)]]);
            }
            db = match resource {
                "order" => db.append_query_results([vec![entity::orders_entity::Model {
                    id: 99,
                    uuid: uuid::Uuid::new_v4(),
                    purchase_id: 88,
                    tenant_id: 8,
                    number: "ORD-99".into(),
                    customer_id: 88,
                    customer_name: "Other customer".into(),
                    customer_tax_id: "12345678901".into(),
                    customer_email: Some("other@example.test".into()),
                    customer_phone: None,
                    status: "paid".into(),
                    payment_status: "paid".into(),
                    subtotal_cents: 1000,
                    discount_cents: 0,
                    shipping_cents: 0,
                    tax_total_cents: 0,
                    total_cents: 1000,
                    coupon_id: None,
                    shipping_snapshot: None,
                    coupon_code: None,
                    placed_at: now,
                    created_at: now,
                    created_by: None,
                    updated_at: now,
                    updated_by: None,
                }]]),
                "purchase" => db.append_query_results([vec![entity::purchase_entity::Model {
                    id: 99,
                    uuid: uuid::Uuid::new_v4(),
                    customer_id: 88,
                    customer_name: "Other customer".into(),
                    customer_tax_id: "12345678901".into(),
                    customer_email: Some("other@example.test".into()),
                    customer_phone: None,
                    currency: "BRL".into(),
                    status: "paid".into(),
                    subtotal_cents: 1000,
                    discount_cents: 0,
                    shipping_cents: 0,
                    tax_total_cents: 0,
                    total_cents: 1000,
                    idempotency_key: "other-key".into(),
                    request_hash: "other-hash".into(),
                    created_at: now,
                    created_by: None,
                    updated_at: now,
                    updated_by: None,
                }]]),
                _ => unreachable!("unknown order resource"),
            };
            let app_state = state(db.into_connection());
            let app = crate::routes::order_routes::order_routes(app_state.clone())
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    crate::commons::tenant_context::tenant_context,
                ))
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
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "GET {path} as {role:?} must not expose another seller/customer resource"
            );
        }
    }
}

#[tokio::test]
async fn remaining_protected_routes_are_registered_and_require_auth() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
    let app_state = state(db);
    let resources = crate::routes::resource_routes::resources_routes(app_state.clone())
        .with_state(app_state.clone());
    let marketing = crate::routes::marketing_routes::marketing_routes(app_state.clone())
        .with_state(app_state.clone());
    let orders =
        crate::routes::order_routes::order_routes(app_state.clone()).with_state(app_state.clone());
    let carts = crate::routes::cart_routes::cart_routes(app_state.clone()).with_state(app_state);

    let resource_routes = [
        ("POST", "/catalog/import"),
        ("POST", "/categories"),
        ("PUT", "/categories/99"),
        ("POST", "/products"),
        ("PUT", "/products/99"),
        ("POST", "/skus"),
        ("PUT", "/skus/99"),
        ("POST", "/sku-attributes"),
        ("PUT", "/sku-attributes/99"),
        ("DELETE", "/sku-attributes/99"),
        ("POST", "/sku-attribute-values"),
        ("PUT", "/sku-attribute-values/99"),
        ("DELETE", "/sku-attribute-values/99"),
        ("POST", "/sku-stocks"),
        ("PUT", "/sku-stocks/99"),
        ("DELETE", "/sku-stocks/99"),
        ("POST", "/resource/avatar/presign"),
    ];
    let marketing_routes = [
        ("POST", "/campaigns"),
        ("PUT", "/campaigns/99"),
        ("POST", "/campaign-targets"),
        ("PUT", "/campaign-targets/99"),
        ("POST", "/coupons"),
        ("PUT", "/coupons/99"),
        ("POST", "/coupon-redemptions"),
    ];
    let order_routes = [
        ("POST", "/purchases"),
        ("POST", "/checkout/quotes/99/shipping-selection"),
        ("POST", "/checkout/quotes"),
        ("GET", "/checkout/payment-config"),
        ("POST", "/purchases/99/payments/submit"),
        ("GET", "/purchases/99/payments/status"),
        ("GET", "/purchases/paged"),
        ("GET", "/purchases/99"),
        ("GET", "/purchases/99/payments"),
        ("GET", "/orders"),
        ("GET", "/orders/paged"),
        ("GET", "/orders/99"),
        ("GET", "/orders/99/status-history"),
        ("GET", "/payments/99/transactions"),
    ];
    let cart_routes = [
        ("GET", "/carts"),
        ("GET", "/carts/paged"),
        ("POST", "/carts"),
        ("GET", "/carts/99"),
        ("PUT", "/carts/99"),
        ("GET", "/cart-items"),
        ("GET", "/cart-items/paged"),
        ("POST", "/cart-items"),
        ("GET", "/cart-items/99"),
        ("PUT", "/cart-items/99"),
    ];

    let checked = assert_routes_require_auth(&resources, &resource_routes).await
        + assert_routes_require_auth(&marketing, &marketing_routes).await
        + assert_routes_require_auth(&orders, &order_routes).await
        + assert_routes_require_auth(&carts, &cart_routes).await;
    assert_eq!(
        checked, 48,
        "all remaining protected method/path pairs must be exercised"
    );
}

#[tokio::test]
async fn authentication_middleware_rejects_expired_bearer_tokens() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET paged customers as {role:?}"
        );
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
            "no_query" => assert!(
                customer_queries.is_empty(),
                "Customer tenantless query was issued"
            ),
            _ => unreachable!("unknown expected scope"),
        }
    }
}

#[tokio::test]
async fn customer_address_collection_routes_scope_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
                "no_query" => assert!(
                    address_queries.is_empty(),
                    "Customer tenantless query was issued"
                ),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }
}

#[tokio::test]
async fn customer_address_create_route_stamps_authenticated_tenant_scope() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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

        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "POST address as {role:?}"
        );
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, true, StatusCode::OK),
        (Role::TenantOwner, Some(7), true, StatusCode::OK),
        (Role::TenantOwner, Some(8), false, StatusCode::NOT_FOUND),
        (Role::TenantUser, Some(7), true, StatusCode::OK),
        (Role::Customer, None, false, StatusCode::NOT_FOUND),
    ];
    let update_body =
        r#"{"tenantId":7,"customerId":19,"recipient":"Updated C001 customer","isDefault":true}"#;

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
            query_results =
                query_results.append_query_results([vec![customer_address_model(90, Some(7), 19)]]);
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, StatusCode::OK, StatusCode::NOT_FOUND),
        (
            Role::TenantOwner,
            Some(7),
            StatusCode::OK,
            StatusCode::NOT_FOUND,
        ),
        (
            Role::TenantOwner,
            Some(8),
            StatusCode::FORBIDDEN,
            StatusCode::FORBIDDEN,
        ),
        (
            Role::TenantUser,
            Some(7),
            StatusCode::FORBIDDEN,
            StatusCode::FORBIDDEN,
        ),
        (
            Role::Customer,
            None,
            StatusCode::FORBIDDEN,
            StatusCode::FORBIDDEN,
        ),
    ];

    for (role, tenant_id, expected_get, expected_put) in cases {
        for (method, expected_status) in [("GET", expected_get), ("PUT", expected_put)] {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([Vec::<BTreeMap<String, Value>>::new()])
                .into_connection();
            let app_state = state(db);
            let app =
                crate::routes::payment_settings_routes::payment_settings_routes(app_state.clone())
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (
            Role::SysAdmin,
            None,
            StatusCode::NOT_FOUND,
            StatusCode::NOT_FOUND,
        ),
        (
            Role::TenantOwner,
            Some(7),
            StatusCode::NOT_FOUND,
            StatusCode::NOT_FOUND,
        ),
        (
            Role::TenantOwner,
            Some(8),
            StatusCode::FORBIDDEN,
            StatusCode::FORBIDDEN,
        ),
        (
            Role::TenantUser,
            Some(7),
            StatusCode::FORBIDDEN,
            StatusCode::FORBIDDEN,
        ),
        (
            Role::Customer,
            None,
            StatusCode::FORBIDDEN,
            StatusCode::FORBIDDEN,
        ),
    ];
    let update_body = serde_json::json!({
        "configuration": business::domain::shipping::ShippingConfig::default()
    })
    .to_string();

    for (role, tenant_id, expected_get, expected_put) in cases {
        for (method, expected_status) in [("GET", expected_get), ("PUT", expected_put)] {
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, StatusCode::OK),
        (Role::TenantOwner, Some(7), StatusCode::OK),
        (Role::TenantOwner, Some(8), StatusCode::NOT_FOUND),
        (Role::TenantUser, Some(7), StatusCode::FORBIDDEN),
        (Role::Customer, None, StatusCode::NOT_FOUND),
    ];
    let listing_row = BTreeMap::from([("listed".to_string(), Value::Bool(Some(true)))]);

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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
            .nest(
                "/tenant",
                crate::routes::tenant_routes::tenant_routes(app_state.clone()),
            )
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
                .nest(
                    "/tenant",
                    crate::routes::tenant_routes::tenant_routes(app_state.clone()),
                )
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
                "no_query" => assert!(
                    tenant_queries.is_empty(),
                    "tenantless Customer query was issued"
                ),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }
}

#[tokio::test]
async fn person_by_id_route_enforces_tenant_scope() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
            .merge(crate::routes::person_routes::person_routes(
                app_state.clone(),
            ))
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
            let table = if is_person {
                "person\""
            } else {
                "person_address\""
            };
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, true, "unrestricted", StatusCode::OK),
        (Role::TenantOwner, Some(7), true, "tenant", StatusCode::OK),
        (
            Role::TenantOwner,
            Some(8),
            false,
            "tenant",
            StatusCode::NOT_FOUND,
        ),
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (
            Role::SysAdmin,
            None,
            StatusCode::FORBIDDEN,
            StatusCode::FORBIDDEN,
        ),
        (
            Role::TenantOwner,
            Some(7),
            StatusCode::FORBIDDEN,
            StatusCode::FORBIDDEN,
        ),
        (
            Role::TenantUser,
            Some(7),
            StatusCode::FORBIDDEN,
            StatusCode::FORBIDDEN,
        ),
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
                json_request(method, path, &authorization, r#"{"cpf":"11144477735"}"#)
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, true, StatusCode::OK, "unrestricted"),
        (
            Role::TenantOwner,
            Some(7),
            false,
            StatusCode::NOT_FOUND,
            "tenant",
        ),
        (Role::TenantOwner, Some(8), true, StatusCode::OK, "tenant"),
        (
            Role::TenantUser,
            Some(7),
            false,
            StatusCode::NOT_FOUND,
            "tenant",
        ),
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
            .nest(
                "/user",
                crate::routes::user_routes::user_routes(app_state.clone()),
            )
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
            .nest(
                "/user",
                crate::routes::user_routes::user_routes(app_state.clone()),
            )
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

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET user list as {role:?}"
        );
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let users: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            users.as_array().unwrap().len(),
            expected_count,
            "role {role:?}"
        );

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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
            .nest(
                "/user",
                crate::routes::user_routes::user_routes(app_state.clone()),
            )
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "no_query"),
        (Role::Customer, None, "no_query"),
    ];

    for (role, tenant_id, expected_scope) in cases {
        let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(1)))]);
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
            .nest(
                "/user",
                crate::routes::user_routes::user_routes(app_state.clone()),
            )
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

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET paged users as {role:?}"
        );
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
            .append_query_results([vec![user_model_for(
                Role::TenantOwner,
                Some(requested_tenant_id),
            )]])
            .append_query_results([vec![person_model(101, 100, Some(requested_tenant_id))]])
            .into_connection();
        let app_state = state(db);
        let app = Router::new()
            .nest(
                "/user",
                crate::routes::user_routes::user_routes(app_state.clone()),
            )
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
            .nest(
                "/user",
                crate::routes::user_routes::user_routes(app_state.clone()),
            )
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
            .nest(
                "/user",
                crate::routes::user_routes::user_routes(app_state.clone()),
            )
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
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

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
            .nest(
                "/user",
                crate::routes::user_routes::user_routes(app_state.clone()),
            )
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

#[tokio::test]
async fn campaign_paged_route_scopes_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "no_query"),
    ];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

    for (role, tenant_id, expected_scope) in cases {
        let mut db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
        if expected_scope != "no_query" {
            db = db
                .append_query_results([vec![count_row.clone()]])
                .append_query_results([Vec::<entity::campaign_entity::Model>::new()]);
        }
        let db = db.into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/campaigns/paged",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET paged campaigns as {role:?}"
        );
        let campaign_queries: Vec<_> = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .filter(|statement| statement.sql.contains("FROM \"campaign\""))
            .collect();
        match expected_scope {
            "unrestricted" => {
                let list_query = campaign_queries.last().expect("SysAdmin campaign query");
                let where_clause = list_query
                    .sql
                    .split_once(" WHERE ")
                    .map(|(_, clause)| clause)
                    .unwrap_or_default();
                assert!(!where_clause.contains("tenant_id"), "{}", list_query.sql);
            }
            "tenant" => {
                let list_query = campaign_queries.last().expect("tenant campaign query");
                assert!(list_query.sql.contains("tenant_id"), "{}", list_query.sql);
            }
            "no_query" => assert!(
                campaign_queries.is_empty(),
                "tenantless Customer query was issued"
            ),
            _ => unreachable!("unknown expected scope"),
        }
    }
}

#[tokio::test]
async fn campaign_by_id_route_enforces_tenant_access() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, StatusCode::OK),
        (Role::TenantOwner, Some(7), StatusCode::NOT_FOUND),
        (Role::TenantOwner, Some(8), StatusCode::OK),
        (Role::TenantUser, Some(7), StatusCode::NOT_FOUND),
        (Role::Customer, None, StatusCode::NOT_FOUND),
    ];
    let now = Utc::now().naive_utc();

    for (role, tenant_id, expected_status) in cases {
        let campaign = entity::campaign_entity::Model {
            id: 99,
            uuid: uuid::Uuid::new_v4(),
            tenant_id: Some(8),
            name: "C001 campaign".into(),
            campaign_type: "percentage".into(),
            value: 10,
            scope: "all".into(),
            starts_at: now,
            ends_at: now,
            active: true,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        };
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([vec![campaign]])
            .into_connection();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/campaigns/99",
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
            "GET campaign 99 as {role:?} for tenant {tenant_id:?}"
        );
    }
}

#[tokio::test]
async fn campaign_list_route_scopes_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "deny"),
    ];

    for (role, tenant_id, expected_scope) in cases {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([Vec::<entity::campaign_entity::Model>::new()])
            .into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/campaigns",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET campaigns as {role:?}"
        );
        let campaign_query = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .find(|statement| statement.sql.contains("FROM \"campaign\""))
            .expect("campaign collection query should execute");
        let where_clause = campaign_query
            .sql
            .split_once(" WHERE ")
            .map(|(_, clause)| clause)
            .unwrap_or_default();
        match expected_scope {
            "unrestricted" => assert!(
                !where_clause.contains("tenant_id"),
                "{}",
                campaign_query.sql
            ),
            "tenant" => assert!(where_clause.contains("tenant_id"), "{}", campaign_query.sql),
            "deny" => assert!(where_clause.contains("1 = 0"), "{}", campaign_query.sql),
            _ => unreachable!("unknown expected scope"),
        }
    }
}

#[tokio::test]
async fn campaign_target_list_route_scopes_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "deny"),
    ];

    for (role, tenant_id, expected_scope) in cases {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([Vec::<entity::campaign_target_entity::Model>::new()])
            .into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/campaign-targets",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET campaign targets as {role:?}"
        );
        let query = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .find(|statement| statement.sql.contains("FROM \"campaign_target\""))
            .expect("campaign-target collection query should execute");
        let where_clause = query
            .sql
            .split_once(" WHERE ")
            .map(|(_, clause)| clause)
            .unwrap_or_default();
        match expected_scope {
            "unrestricted" => assert!(!where_clause.contains("tenant_id"), "{}", query.sql),
            "tenant" => assert!(where_clause.contains("tenant_id"), "{}", query.sql),
            "deny" => assert!(where_clause.contains("1 = 0"), "{}", query.sql),
            _ => unreachable!("unknown expected scope"),
        }
    }
}

#[tokio::test]
async fn campaign_target_paged_route_scopes_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "no_query"),
    ];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

    for (role, tenant_id, expected_scope) in cases {
        let mut db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
        if expected_scope != "no_query" {
            db = db
                .append_query_results([vec![count_row.clone()]])
                .append_query_results([Vec::<entity::campaign_target_entity::Model>::new()]);
        }
        let db = db.into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/campaign-targets/paged",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET paged campaign targets as {role:?}"
        );
        let queries: Vec<_> = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .filter(|statement| statement.sql.contains("FROM \"campaign_target\""))
            .collect();
        match expected_scope {
            "unrestricted" => {
                let list_query = queries.last().expect("SysAdmin campaign-target query");
                let where_clause = list_query
                    .sql
                    .split_once(" WHERE ")
                    .map(|(_, clause)| clause)
                    .unwrap_or_default();
                assert!(!where_clause.contains("tenant_id"), "{}", list_query.sql);
            }
            "tenant" => {
                let list_query = queries.last().expect("tenant campaign-target query");
                assert!(list_query.sql.contains("tenant_id"), "{}", list_query.sql);
            }
            "no_query" => assert!(queries.is_empty(), "tenantless Customer query was issued"),
            _ => unreachable!("unknown expected scope"),
        }
    }
}

#[tokio::test]
async fn coupon_list_route_scopes_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "deny"),
    ];

    for (role, tenant_id, expected_scope) in cases {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([Vec::<entity::coupon_entity::Model>::new()])
            .into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/coupons",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, "GET coupons as {role:?}");
        let query = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .find(|statement| statement.sql.contains("FROM \"coupon\""))
            .expect("coupon collection query should execute");
        let where_clause = query
            .sql
            .split_once(" WHERE ")
            .map(|(_, clause)| clause)
            .unwrap_or_default();
        match expected_scope {
            "unrestricted" => assert!(!where_clause.contains("tenant_id"), "{}", query.sql),
            "tenant" => assert!(where_clause.contains("tenant_id"), "{}", query.sql),
            "deny" => assert!(where_clause.contains("1 = 0"), "{}", query.sql),
            _ => unreachable!("unknown expected scope"),
        }
    }
}

#[tokio::test]
async fn coupon_paged_route_scopes_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "no_query"),
    ];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

    for (role, tenant_id, expected_scope) in cases {
        let mut db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
        if expected_scope != "no_query" {
            db = db
                .append_query_results([vec![count_row.clone()]])
                .append_query_results([Vec::<entity::coupon_entity::Model>::new()]);
        }
        let db = db.into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/coupons/paged",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET paged coupons as {role:?}"
        );
        let queries: Vec<_> = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .filter(|statement| statement.sql.contains("FROM \"coupon\""))
            .collect();
        match expected_scope {
            "unrestricted" => {
                let list_query = queries.last().expect("SysAdmin coupon query");
                let where_clause = list_query
                    .sql
                    .split_once(" WHERE ")
                    .map(|(_, clause)| clause)
                    .unwrap_or_default();
                assert!(!where_clause.contains("tenant_id"), "{}", list_query.sql);
            }
            "tenant" => {
                let list_query = queries.last().expect("tenant coupon query");
                assert!(list_query.sql.contains("tenant_id"), "{}", list_query.sql);
            }
            "no_query" => assert!(queries.is_empty(), "tenantless Customer query was issued"),
            _ => unreachable!("unknown expected scope"),
        }
    }
}

#[tokio::test]
async fn coupon_by_id_route_enforces_tenant_access() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, StatusCode::OK),
        (Role::TenantOwner, Some(7), StatusCode::NOT_FOUND),
        (Role::TenantOwner, Some(8), StatusCode::OK),
        (Role::TenantUser, Some(7), StatusCode::NOT_FOUND),
        (Role::Customer, None, StatusCode::NOT_FOUND),
    ];
    let now = Utc::now().naive_utc();

    for (role, tenant_id, expected_status) in cases {
        let coupon = entity::coupon_entity::Model {
            id: 99,
            uuid: uuid::Uuid::new_v4(),
            tenant_id: Some(8),
            code: "C001-TEST".into(),
            campaign_id: Some(10),
            coupon_type: "percentage".into(),
            value: 10,
            min_order_cents: None,
            max_uses: None,
            max_uses_per_customer: None,
            starts_at: None,
            expires_at: None,
            active: true,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        };
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([vec![coupon]])
            .into_connection();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/coupons/99",
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
            "GET coupon 99 as {role:?} for tenant {tenant_id:?}"
        );
    }
}

#[tokio::test]
async fn coupon_redemption_list_route_scopes_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "deny"),
    ];

    for (role, tenant_id, expected_scope) in cases {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([Vec::<entity::coupon_redemption_entity::Model>::new()])
            .into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/coupon-redemptions",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET redemptions as {role:?}"
        );
        let query = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .find(|statement| statement.sql.contains("FROM \"coupon_redemption\""))
            .expect("redemption collection query should execute");
        let where_clause = query
            .sql
            .split_once(" WHERE ")
            .map(|(_, clause)| clause)
            .unwrap_or_default();
        match expected_scope {
            "unrestricted" => assert!(!where_clause.contains("tenant_id"), "{}", query.sql),
            "tenant" => assert!(where_clause.contains("tenant_id"), "{}", query.sql),
            "deny" => assert!(where_clause.contains("1 = 0"), "{}", query.sql),
            _ => unreachable!("unknown expected scope"),
        }
    }
}

#[tokio::test]
async fn coupon_redemption_paged_route_scopes_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "no_query"),
    ];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

    for (role, tenant_id, expected_scope) in cases {
        let mut db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
        if expected_scope != "no_query" {
            db = db
                .append_query_results([vec![count_row.clone()]])
                .append_query_results([Vec::<entity::coupon_redemption_entity::Model>::new()]);
        }
        let db = db.into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/coupon-redemptions/paged",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET paged redemptions as {role:?}"
        );
        let queries: Vec<_> = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .filter(|statement| statement.sql.contains("FROM \"coupon_redemption\""))
            .collect();
        match expected_scope {
            "unrestricted" => {
                let list_query = queries.last().expect("SysAdmin redemption query");
                let where_clause = list_query
                    .sql
                    .split_once(" WHERE ")
                    .map(|(_, clause)| clause)
                    .unwrap_or_default();
                assert!(!where_clause.contains("tenant_id"), "{}", list_query.sql);
            }
            "tenant" => {
                let list_query = queries.last().expect("tenant redemption query");
                assert!(list_query.sql.contains("tenant_id"), "{}", list_query.sql);
            }
            "no_query" => assert!(queries.is_empty(), "tenantless Customer query was issued"),
            _ => unreachable!("unknown expected scope"),
        }
    }
}

#[tokio::test]
async fn coupon_redemption_by_id_route_enforces_tenant_access() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, StatusCode::OK),
        (Role::TenantOwner, Some(7), StatusCode::NOT_FOUND),
        (Role::TenantOwner, Some(8), StatusCode::OK),
        (Role::TenantUser, Some(7), StatusCode::NOT_FOUND),
        (Role::Customer, None, StatusCode::NOT_FOUND),
    ];
    let now = Utc::now().naive_utc();

    for (role, tenant_id, expected_status) in cases {
        let redemption = entity::coupon_redemption_entity::Model {
            id: 99,
            uuid: uuid::Uuid::new_v4(),
            tenant_id: Some(8),
            coupon_id: 10,
            order_id: 20,
            customer_id: 30,
            created_at: now,
            created_by: None,
        };
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([vec![redemption]])
            .into_connection();
        let app_state = state(db);
        let app = crate::routes::marketing_routes::marketing_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/coupon-redemptions/99",
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
            "GET redemption 99 as {role:?} for tenant {tenant_id:?}"
        );
    }
}

#[tokio::test]
async fn province_list_route_is_available_to_authenticated_roles() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None),
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];

    for (role, tenant_id) in cases {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([Vec::<entity::province_entity::Model>::new()])
            .into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::resource_routes::resources_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/province?countryCode=BR",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET provinces as {role:?}"
        );
        let query = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .find(|statement| statement.sql.contains("FROM \"province\""))
            .expect("province lookup query should execute");
        assert!(query.sql.contains("country_code"), "{}", query.sql);
        assert!(!query.sql.contains("tenant_id"), "{}", query.sql);
    }
}

#[tokio::test]
async fn province_write_routes_are_sysadmin_only() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None),
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];
    let payload = r#"{"acronym":"SP","name":"Sao Paulo","countryCode":"BR","ibgeCode":"35"}"#;

    for (role, tenant_id) in cases {
        for (method, path) in [
            ("POST", "/province"),
            ("PUT", "/province/1"),
            ("POST", "/province/import"),
        ] {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .into_connection();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
                .with_state(app_state);
            let authorization = format!(
                "Bearer {}",
                token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
            );
            let request = if path == "/province/import" {
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("authorization", authorization)
                    .header("content-type", "multipart/form-data; boundary=c001")
                    .body(Body::from("--c001--\r\n"))
                    .unwrap()
            } else {
                json_request(method, path, &authorization, payload)
            };
            let response = app.oneshot(request).await.unwrap();

            if role == Role::SysAdmin {
                assert_ne!(
                    response.status(),
                    StatusCode::FORBIDDEN,
                    "SysAdmin should pass the role guard for {method} {path}"
                );
            } else {
                assert_eq!(
                    response.status(),
                    StatusCode::FORBIDDEN,
                    "{role:?} should be denied for {method} {path}"
                );
            }
        }
    }
}

#[tokio::test]
async fn city_collection_routes_are_available_to_authenticated_roles() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None),
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];
    let paths = [
        ("/cities", false),
        ("/cities/by-province/7", true),
        ("/city/by-province/7", true),
    ];

    for (role, tenant_id) in roles {
        for (path, filters_by_province) in paths {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([Vec::<entity::city_entity::Model>::new()])
                .into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
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
            let query = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .find(|statement| statement.sql.contains("FROM \"city\""))
                .expect("city collection query should execute");
            let where_clause = query
                .sql
                .split_once(" WHERE ")
                .map(|(_, clause)| clause)
                .unwrap_or_default();
            assert!(!query.sql.contains("tenant_id"), "{}", query.sql);
            assert_eq!(
                where_clause.contains("province_id"),
                filters_by_province,
                "{path} query: {}",
                query.sql
            );
        }
    }
}

#[tokio::test]
async fn city_paged_routes_are_available_to_authenticated_roles() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None),
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];
    let paths = ["/cities/paged", "/city/paged"];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

    for (role, tenant_id) in roles {
        for path in paths {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![count_row.clone()]])
                .append_query_results([Vec::<entity::city_entity::Model>::new()])
                .into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
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
            let queries: Vec<_> = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .filter(|statement| statement.sql.contains("FROM \"city\""))
                .collect();
            assert_eq!(queries.len(), 2, "{path} should count and fetch city rows");
            assert!(queries.iter().all(|query| !query.sql.contains("tenant_id")));
        }
    }
}

#[tokio::test]
async fn city_by_id_route_is_available_to_authenticated_roles() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None),
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];

    for (role, tenant_id) in roles {
        let city = entity::city_entity::Model {
            id: 99,
            uuid: uuid::Uuid::new_v4(),
            province_id: 7,
            name: "C001 city".into(),
            ibge_code: Some("1234567".into()),
        };
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([vec![city]])
            .into_connection();
        let app_state = state(db);
        let app = crate::routes::resource_routes::resources_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/city/99",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, "GET city as {role:?}");
    }
}

#[tokio::test]
async fn city_write_routes_are_sysadmin_only() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None),
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];
    let payload = r#"{"provinceId":7,"name":"C001 city","ibgeCode":"1234567"}"#;

    for (role, tenant_id) in cases {
        for (method, path) in [
            ("POST", "/city"),
            ("PUT", "/city/1"),
            ("POST", "/city/import"),
        ] {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .into_connection();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
                .with_state(app_state);
            let authorization = format!(
                "Bearer {}",
                token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
            );
            let request = if path == "/city/import" {
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("authorization", authorization)
                    .header("content-type", "multipart/form-data; boundary=c001")
                    .body(Body::from("--c001--\r\n"))
                    .unwrap()
            } else {
                json_request(method, path, &authorization, payload)
            };
            let response = app.oneshot(request).await.unwrap();

            if role == Role::SysAdmin {
                assert_ne!(
                    response.status(),
                    StatusCode::FORBIDDEN,
                    "SysAdmin should pass the role guard for {method} {path}"
                );
            } else {
                assert_eq!(
                    response.status(),
                    StatusCode::FORBIDDEN,
                    "{role:?} should be denied for {method} {path}"
                );
            }
        }
    }
}

#[tokio::test]
async fn province_read_routes_are_available_to_authenticated_roles() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None),
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

    for (role, tenant_id) in roles {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([vec![count_row.clone()]])
            .append_query_results([Vec::<entity::province_entity::Model>::new()])
            .into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::resource_routes::resources_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/province/paged",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET paged provinces as {role:?}"
        );
        let queries: Vec<_> = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .filter(|statement| statement.sql.contains("FROM \"province\""))
            .collect();
        assert_eq!(
            queries.len(),
            2,
            "province page should count and fetch rows"
        );
        assert!(queries.iter().all(|query| !query.sql.contains("tenant_id")));

        let province = entity::province_entity::Model {
            id: 7,
            uuid: uuid::Uuid::new_v4(),
            acronym: "SP".into(),
            name: "Sao Paulo".into(),
            country_code: "BR".into(),
            ibge_code: Some("35".into()),
        };
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([vec![province]])
            .into_connection();
        let app_state = state(db);
        let app = crate::routes::resource_routes::resources_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/province/7",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET province by ID as {role:?}"
        );
    }
}

#[tokio::test]
async fn product_category_collection_routes_scope_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "deny"),
    ];
    let paths = [
        "/product-categories",
        "/product-categories/by-product/99",
        "/product-categories/by-category/88",
    ];
    let now = Utc::now().naive_utc();

    for (role, tenant_id, expected_scope) in roles {
        for path in paths {
            let row = entity::product_category_entity::Model {
                id: 1,
                uuid: uuid::Uuid::new_v4(),
                tenant_id: Some(8),
                product_id: 99,
                category_id: 88,
                is_primary: true,
                created_at: now,
                created_by: None,
            };
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![row]])
                .into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
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
            let query = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .find(|statement| statement.sql.contains("FROM \"product_category\""))
                .expect("product-category query should execute");
            let where_clause = query
                .sql
                .split_once(" WHERE ")
                .map(|(_, clause)| clause)
                .unwrap_or_default();
            match expected_scope {
                "unrestricted" => assert!(!where_clause.contains("tenant_id"), "{}", query.sql),
                "tenant" => assert!(where_clause.contains("tenant_id"), "{}", query.sql),
                "deny" => assert!(where_clause.contains("1 = 0"), "{}", query.sql),
                _ => unreachable!("unknown expected scope"),
            }
        }

        let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);
        let mut db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
        if expected_scope != "deny" {
            db = db
                .append_query_results([vec![count_row]])
                .append_query_results([Vec::<entity::product_category_entity::Model>::new()]);
        }
        let db = db.into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::resource_routes::resources_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/product-categories/paged",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "paged as {role:?}");
        let queries: Vec<_> = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .filter(|statement| statement.sql.contains("FROM \"product_category\""))
            .collect();
        if expected_scope == "deny" {
            assert!(queries.is_empty(), "tenantless Customer query was issued");
        } else {
            assert_eq!(queries.len(), 2, "paged route should count and fetch rows");
            let list_query = queries.last().unwrap();
            match expected_scope {
                "unrestricted" => assert!(
                    !list_query
                        .sql
                        .contains(" WHERE \"product_category\".\"tenant_id\""),
                    "{}",
                    list_query.sql
                ),
                "tenant" => assert!(list_query.sql.contains("tenant_id"), "{}", list_query.sql),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }
}

#[tokio::test]
async fn product_category_by_id_enforces_tenant_access() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, StatusCode::OK),
        (Role::TenantOwner, Some(7), StatusCode::NOT_FOUND),
        (Role::TenantOwner, Some(8), StatusCode::OK),
        (Role::TenantUser, Some(7), StatusCode::NOT_FOUND),
        (Role::Customer, None, StatusCode::NOT_FOUND),
    ];
    let now = Utc::now().naive_utc();

    for (role, tenant_id, expected_status) in cases {
        let row = entity::product_category_entity::Model {
            id: 99,
            uuid: uuid::Uuid::new_v4(),
            tenant_id: Some(8),
            product_id: 99,
            category_id: 88,
            is_primary: true,
            created_at: now,
            created_by: None,
        };
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([vec![row]])
            .into_connection();
        let app_state = state(db);
        let app = crate::routes::resource_routes::resources_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/product-categories/99",
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
            "GET product category as {role:?} for tenant {tenant_id:?}"
        );
    }
}

#[tokio::test]
async fn product_routes_scope_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "deny"),
    ];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

    for (role, tenant_id, expected_scope) in roles {
        for (path, paged) in [("/products", false), ("/products/paged", true)] {
            let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
            if paged && expected_scope == "deny" {
            } else {
                if paged {
                    db = db.append_query_results([vec![count_row.clone()]]);
                }
                db = db.append_query_results([Vec::<entity::product_entity::Model>::new()]);
            }
            let db = db.into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
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

            let queries: Vec<_> = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .filter(|statement| statement.sql.contains("FROM \"product\""))
                .collect();
            if paged && expected_scope == "deny" {
                assert!(
                    queries.is_empty(),
                    "tenantless Customer product query was issued"
                );
                continue;
            }
            let query = if paged {
                assert_eq!(queries.len(), 2, "paged products should count and fetch");
                &queries[1]
            } else {
                assert_eq!(queries.len(), 1);
                &queries[0]
            };
            let where_clause = query
                .sql
                .split_once(" WHERE ")
                .map(|(_, clause)| clause)
                .unwrap_or_default();
            match expected_scope {
                "unrestricted" => assert!(!where_clause.contains("tenant_id"), "{}", query.sql),
                "tenant" => assert!(where_clause.contains("tenant_id"), "{}", query.sql),
                "deny" => assert!(where_clause.contains("1 = 0"), "{}", query.sql),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }
}

#[tokio::test]
async fn sku_routes_scope_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "deny"),
    ];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

    for (role, tenant_id, expected_scope) in roles {
        for (path, paged) in [("/skus", false), ("/skus/paged", true)] {
            let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
            if !(paged && expected_scope == "deny") {
                if paged {
                    db = db.append_query_results([vec![count_row.clone()]]);
                }
                db = db.append_query_results([Vec::<entity::sku_entity::Model>::new()]);
            }
            let db = db.into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
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

            let queries: Vec<_> = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .filter(|statement| statement.sql.contains("FROM \"sku\""))
                .collect();
            if paged && expected_scope == "deny" {
                assert!(
                    queries.is_empty(),
                    "tenantless Customer SKU query was issued"
                );
                continue;
            }
            let query = if paged {
                assert_eq!(queries.len(), 2, "paged SKUs should count and fetch");
                &queries[1]
            } else {
                assert_eq!(queries.len(), 1);
                &queries[0]
            };
            let where_clause = query
                .sql
                .split_once(" WHERE ")
                .map(|(_, clause)| clause)
                .unwrap_or_default();
            match expected_scope {
                "unrestricted" => assert!(!where_clause.contains("tenant_id"), "{}", query.sql),
                "tenant" => assert!(where_clause.contains("tenant_id"), "{}", query.sql),
                "deny" => assert!(where_clause.contains("1 = 0"), "{}", query.sql),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }
}

#[tokio::test]
async fn sku_stock_collection_routes_scope_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "deny"),
    ];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

    for (role, tenant_id, expected_scope) in roles {
        for (path, paged) in [("/sku-stocks", false), ("/sku-stocks/paged", true)] {
            let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
            if !(paged && expected_scope == "deny") {
                if paged {
                    db = db.append_query_results([vec![count_row.clone()]]);
                }
                db = db.append_query_results([Vec::<entity::sku_stock_entity::Model>::new()]);
            }
            let db = db.into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
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

            let queries: Vec<_> = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .filter(|statement| statement.sql.contains("FROM \"sku_stock\""))
                .collect();
            if paged && expected_scope == "deny" {
                assert!(
                    queries.is_empty(),
                    "tenantless Customer stock query was issued"
                );
                continue;
            }
            let query = if paged {
                assert_eq!(queries.len(), 2, "paged stock should count and fetch");
                &queries[1]
            } else {
                assert_eq!(queries.len(), 1);
                &queries[0]
            };
            let where_clause = query
                .sql
                .split_once(" WHERE ")
                .map(|(_, clause)| clause)
                .unwrap_or_default();
            match expected_scope {
                "unrestricted" => assert!(!where_clause.contains("tenant_id"), "{}", query.sql),
                "tenant" => assert!(where_clause.contains("tenant_id"), "{}", query.sql),
                "deny" => assert!(where_clause.contains("1 = 0"), "{}", query.sql),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }
}

#[tokio::test]
async fn profile_put_aliases_update_authenticated_person_for_all_roles() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None),
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];
    let paths = ["/resource/profile", "/profile"];
    let payload =
        r#"{"userId":999,"firstName":"  Updated Name  ","surname":"  Updated Surname  "}"#;

    for (role, tenant_id) in roles {
        for path in paths {
            let mut updated_person = person_model(77, 42, tenant_id);
            updated_person.first_name = "Updated Name".into();
            updated_person.surname = Some("Updated Surname".into());
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![person_model(77, 42, tenant_id)]])
                .append_query_results([vec![updated_person]])
                .into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
                .with_state(app_state);
            let response = app
                .oneshot(json_request(
                    "PUT",
                    path,
                    &format!(
                        "Bearer {}",
                        token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                    ),
                    payload,
                ))
                .await
                .unwrap();

            let status = response.status();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let statements: Vec<_> = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .into_iter()
                .map(|statement| statement.sql)
                .collect();
            assert_eq!(
                status,
                StatusCode::OK,
                "PUT {path} as {role:?}: {}; SQL: {statements:?}",
                String::from_utf8_lossy(&body)
            );
            let person: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(person["id"], 77);
            assert_eq!(person["userId"], 42);
            assert_eq!(person["firstName"], "Updated Name");
            assert_eq!(person["surname"], "Updated Surname");
        }
    }
}

#[tokio::test]
async fn profile_get_aliases_return_authenticated_person_for_all_roles() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None),
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];
    let paths = ["/resource/profile", "/resource", "/resource/me", "/profile"];

    for (role, tenant_id) in roles {
        for path in paths {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![person_model(77, 42, tenant_id)]])
                .append_query_results([Vec::<entity::person_address_entity::Model>::new()])
                .into_connection();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
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
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(profile["user"]["role"], role.to_string());
            assert_eq!(profile["person"]["userId"], 42);
            assert_eq!(profile["person"]["id"], 77);
            assert!(profile["addresses"].as_array().unwrap().is_empty());
        }
    }
}

#[tokio::test]
async fn product_image_list_enforces_product_tenant_access() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let cases = [
        (Role::SysAdmin, None, StatusCode::OK),
        (Role::TenantOwner, Some(7), StatusCode::FORBIDDEN),
        (Role::TenantOwner, Some(8), StatusCode::OK),
        (Role::TenantUser, Some(7), StatusCode::FORBIDDEN),
        (Role::Customer, None, StatusCode::FORBIDDEN),
    ];
    let now = Utc::now().naive_utc();

    for (role, tenant_id, expected_status) in cases {
        let product = entity::product_entity::Model {
            id: 99,
            uuid: uuid::Uuid::new_v4(),
            tenant_id: Some(8),
            name: "C001 product".into(),
            slug: "c001-product".into(),
            description: None,
            brand: None,
            active: true,
            ncm: "12345678".into(),
            cest: None,
            origem_mercadoria: 0,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        };
        let mut db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([vec![product]]);
        if expected_status == StatusCode::OK {
            db = db.append_query_results([Vec::<entity::product_image_entity::Model>::new()]);
        }
        let app_state = state(db.into_connection());
        let app = crate::routes::resource_routes::resources_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/products/99/images",
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
            "GET product images as {role:?} for tenant {tenant_id:?}"
        );
    }
}

#[tokio::test]
async fn product_image_mutations_require_product_access_before_storage() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::TenantOwner, Some(7)),
        (Role::TenantUser, Some(7)),
        (Role::Customer, None),
    ];
    let payload =
        r#"{"images":[{"originalFilename":"image.jpg","mimeType":"image/jpeg","sizeBytes":1024}]}"#;

    for (role, tenant_id) in roles {
        for (method, path) in [
            ("POST", "/products/99/images/presign"),
            ("DELETE", "/products/99/images/101"),
            ("PUT", "/products/99/images/101/primary"),
        ] {
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([Vec::<entity::product_entity::Model>::new()])
                .into_connection();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
                .with_state(app_state);
            let authorization = format!(
                "Bearer {}",
                token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
            );
            let request = if method == "POST" {
                json_request(method, path, &authorization, payload)
            } else {
                request(method, path, Some(&authorization))
            };
            let response = app.oneshot(request).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "{role:?} should not operate on inaccessible product via {method} {path}"
            );
        }
    }
}

#[tokio::test]
async fn sku_stock_by_id_and_sku_enforce_tenant_access() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None, StatusCode::OK),
        (Role::TenantOwner, Some(7), StatusCode::NOT_FOUND),
        (Role::TenantOwner, Some(8), StatusCode::OK),
        (Role::TenantUser, Some(7), StatusCode::NOT_FOUND),
        (Role::Customer, None, StatusCode::NOT_FOUND),
    ];
    let now = Utc::now().naive_utc();

    for (role, tenant_id, expected_status) in roles {
        for path in ["/sku-stocks/99", "/sku-stocks/by-sku/99"] {
            let stock = entity::sku_stock_entity::Model {
                id: 99,
                uuid: uuid::Uuid::new_v4(),
                tenant_id: Some(8),
                sku_id: 99,
                warehouse_id: Some(1),
                quantity: 10,
                reserved: 0,
                created_at: now,
                created_by: None,
                updated_at: now,
                updated_by: None,
            };
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![stock]])
                .into_connection();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
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
            assert_eq!(
                response.status(),
                expected_status,
                "GET {path} as {role:?} for tenant {tenant_id:?}"
            );
        }
    }
}

#[tokio::test]
async fn sku_attribute_list_routes_scope_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "deny"),
    ];
    let routes = [
        ("/sku-attributes", false),
        ("/sku-attributes/paged", true),
        ("/sku-attributes/by-sku/99", false),
        ("/sku-attributes/by-product/99", false),
        ("/sku-attribute-values", false),
        ("/sku-attribute-values/paged", true),
        ("/sku-attribute-values/by-sku/99", false),
    ];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);
    let now = Utc::now().naive_utc();

    for (role, tenant_id, expected_scope) in roles {
        for (path, paged) in routes {
            let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
            if !(paged && expected_scope == "deny") {
                if paged {
                    db = db.append_query_results([vec![count_row.clone()]]);
                }
                let row = entity::sku_attribute_value_entity::Model {
                    id: 99,
                    uuid: uuid::Uuid::new_v4(),
                    tenant_id: Some(8),
                    product_id: 44,
                    sku_id: 99,
                    product_attribute_id: 55,
                    attribute_id: 66,
                    attribute_value_id: 77,
                    created_at: now,
                    created_by: None,
                    updated_at: now,
                    updated_by: None,
                };
                db = db.append_query_results([vec![row]]);
            }
            let db = db.into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
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

            let queries: Vec<_> = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .filter(|statement| statement.sql.contains("FROM \"sku_attribute_value\""))
                .collect();
            if paged && expected_scope == "deny" {
                assert!(
                    queries.is_empty(),
                    "tenantless Customer query was issued for {path}"
                );
                continue;
            }
            let query = if paged {
                assert_eq!(queries.len(), 2, "{path} should count and fetch rows");
                &queries[1]
            } else {
                assert_eq!(queries.len(), 1, "{path} should execute one query");
                &queries[0]
            };
            let where_clause = query
                .sql
                .split_once(" WHERE ")
                .map(|(_, clause)| clause)
                .unwrap_or_default();
            match expected_scope {
                "unrestricted" => assert!(!where_clause.contains("tenant_id"), "{}", query.sql),
                "tenant" => assert!(where_clause.contains("tenant_id"), "{}", query.sql),
                "deny" => assert!(where_clause.contains("1 = 0"), "{}", query.sql),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }
}

#[tokio::test]
async fn sku_attribute_id_aliases_enforce_tenant_access() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None, StatusCode::OK),
        (Role::TenantOwner, Some(7), StatusCode::NOT_FOUND),
        (Role::TenantOwner, Some(8), StatusCode::OK),
        (Role::TenantUser, Some(7), StatusCode::NOT_FOUND),
        (Role::Customer, None, StatusCode::NOT_FOUND),
    ];
    let now = Utc::now().naive_utc();

    for (role, tenant_id, expected_status) in roles {
        for path in ["/sku-attributes/99", "/sku-attribute-values/99"] {
            let row = entity::sku_attribute_value_entity::Model {
                id: 99,
                uuid: uuid::Uuid::new_v4(),
                tenant_id: Some(8),
                product_id: 44,
                sku_id: 99,
                product_attribute_id: 55,
                attribute_id: 66,
                attribute_value_id: 77,
                created_at: now,
                created_by: None,
                updated_at: now,
                updated_by: None,
            };
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
                .append_query_results([vec![row]])
                .into_connection();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
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
            assert_eq!(
                response.status(),
                expected_status,
                "GET {path} as {role:?} for tenant {tenant_id:?}"
            );
        }
    }
}

#[tokio::test]
async fn catalog_attribute_routes_scope_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "deny"),
    ];
    let routes = [
        ("/catalog-attributes", "catalog_attribute", false),
        ("/catalog-attributes/paged", "catalog_attribute", true),
        (
            "/catalog-attribute-values",
            "catalog_attribute_value",
            false,
        ),
        (
            "/catalog-attribute-values/paged",
            "catalog_attribute_value",
            true,
        ),
        ("/product-attributes", "product_attribute", false),
        ("/product-attributes/paged", "product_attribute", true),
    ];
    let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);

    for (role, tenant_id, expected_scope) in roles {
        for (path, table, paged) in routes {
            let mut db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
            if paged && expected_scope == "deny" {
                // Tenantless paged reads return before querying the resource table.
            } else {
                if paged {
                    db = db.append_query_results([vec![count_row.clone()]]);
                }
                db = match table {
                    "catalog_attribute" => db.append_query_results([Vec::<
                        entity::catalog_attribute_entity::Model,
                    >::new()]),
                    "catalog_attribute_value" => db
                        .append_query_results([
                            Vec::<entity::catalog_attribute_value_entity::Model>::new(),
                        ]),
                    "product_attribute" => db.append_query_results([Vec::<
                        entity::product_attribute_entity::Model,
                    >::new()]),
                    _ => unreachable!("unknown catalog attribute table"),
                };
            }

            let db = db.into_connection();
            let db_for_log = db.clone();
            let app_state = state(db);
            let app = crate::routes::resource_routes::resources_routes(app_state.clone())
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

            let queries: Vec<_> = db_for_log
                .into_transaction_log()
                .into_iter()
                .flat_map(|transaction| transaction.statements().to_vec())
                .filter(|statement| statement.sql.contains(&format!("FROM \"{table}\"")))
                .collect();
            if paged && expected_scope == "deny" {
                assert!(
                    queries.is_empty(),
                    "tenantless Customer query was issued for {path}"
                );
                continue;
            }

            let resource_query = if paged {
                assert_eq!(queries.len(), 2, "{path} should count and fetch rows");
                &queries[1]
            } else {
                assert_eq!(queries.len(), 1, "{path} should execute one query");
                &queries[0]
            };
            let where_clause = resource_query
                .sql
                .split_once(" WHERE ")
                .map(|(_, clause)| clause)
                .unwrap_or_default();
            match expected_scope {
                "unrestricted" => assert!(
                    !where_clause.contains("tenant_id"),
                    "{}",
                    resource_query.sql
                ),
                "tenant" => assert!(where_clause.contains("tenant_id"), "{}", resource_query.sql),
                "deny" => assert!(where_clause.contains("1 = 0"), "{}", resource_query.sql),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }
}

#[tokio::test]
async fn category_collection_and_paged_routes_scope_queries_by_role() {
    unsafe {
        std::env::set_var("ACCESS_TOKEN_SECRET", TEST_SECRET);
    }

    let roles = [
        (Role::SysAdmin, None, "unrestricted"),
        (Role::TenantOwner, Some(7), "tenant"),
        (Role::TenantUser, Some(7), "tenant"),
        (Role::Customer, None, "deny"),
    ];
    let now = Utc::now().naive_utc();

    for (role, tenant_id, expected_scope) in roles {
        let category = entity::category_entity::Model {
            id: 11,
            uuid: uuid::Uuid::new_v4(),
            tenant_id: Some(8),
            name: "C001 category".into(),
            slug: "c001-category".into(),
            parent_id: None,
            active: true,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        };
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]])
            .append_query_results([vec![category.clone()]])
            .into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::resource_routes::resources_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/categories",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET categories as {role:?}"
        );
        let query = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .find(|statement| statement.sql.contains("FROM \"category\""))
            .expect("category collection query should execute");
        let where_clause = query
            .sql
            .split_once(" WHERE ")
            .map(|(_, clause)| clause)
            .unwrap_or_default();
        match expected_scope {
            "unrestricted" => assert!(!where_clause.contains("tenant_id"), "{}", query.sql),
            "tenant" => assert!(where_clause.contains("tenant_id"), "{}", query.sql),
            "deny" => assert!(where_clause.contains("1 = 0"), "{}", query.sql),
            _ => unreachable!("unknown expected scope"),
        }

        let count_row = BTreeMap::from([("num_items".to_string(), Value::BigInt(Some(0)))]);
        let mut db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model_for(role.clone(), tenant_id)]]);
        if expected_scope != "deny" {
            db = db
                .append_query_results([vec![count_row]])
                .append_query_results([Vec::<entity::category_entity::Model>::new()]);
        }
        let db = db.into_connection();
        let db_for_log = db.clone();
        let app_state = state(db);
        let app = crate::routes::resource_routes::resources_routes(app_state.clone())
            .with_state(app_state);
        let response = app
            .oneshot(request(
                "GET",
                "/categories/paged",
                Some(&format!(
                    "Bearer {}",
                    token_for(role.clone(), tenant_id, Utc::now().timestamp() + 3600)
                )),
            ))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "paged categories as {role:?}"
        );
        let queries: Vec<_> = db_for_log
            .into_transaction_log()
            .into_iter()
            .flat_map(|transaction| transaction.statements().to_vec())
            .filter(|statement| statement.sql.contains("FROM \"category\""))
            .collect();
        if expected_scope == "deny" {
            assert!(
                queries.is_empty(),
                "tenantless Customer category query was issued"
            );
        } else {
            assert_eq!(queries.len(), 2);
            let list_query = queries.last().unwrap();
            match expected_scope {
                "unrestricted" => assert!(
                    !list_query.sql.contains(" WHERE \"category\".\"tenant_id\""),
                    "{}",
                    list_query.sql
                ),
                "tenant" => assert!(list_query.sql.contains("tenant_id"), "{}", list_query.sql),
                _ => unreachable!("unknown expected scope"),
            }
        }
    }
}
