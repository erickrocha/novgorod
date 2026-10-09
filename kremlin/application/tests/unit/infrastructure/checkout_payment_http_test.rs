use crate::{
    AppState,
    commons::{i18n::Locale, tenant_context::TenantContext},
    endpoints::{
        checkout_payment_endpoint::{status, submit},
        checkout_quote_endpoint::{quote, select_shipping},
        orders_endpoint::{
            create_purchase, get_by_id, get_purchase, history, payments, transactions,
        },
    },
    infrastructure::{
        correios::Correios, payment_credentials::PaymentKeyRing,
        shipping_credentials::ShippingKeyRing,
    },
};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
    routing::{get, post},
};
use business::{
    domain::{enums::Role, user::User},
    sea_orm::{ConnectOptions, ConnectionTrait, Database, DbBackend, DbConn, Statement},
};
use migration::{Migrator, MigratorTrait};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

async fn database() -> anyhow::Result<(DbConn, DbConn, String)> {
    let url = std::env::var("KREMLIN_TEST_DATABASE_URL")?;
    anyhow::ensure!(
        url.split('?').next().unwrap_or_default().ends_with("_test"),
        "test database name must end in _test"
    );
    let root = Database::connect(url.clone()).await?;
    let schema = format!("checkout_payment_test_{}", uuid::Uuid::new_v4().simple());
    root.execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await?;
    let mut options = ConnectOptions::new(url);
    options
        .set_schema_search_path(&schema)
        .sqlx_logging(false)
        .max_connections(8);
    let db = Database::connect(options).await?;
    Ok((root, db, schema))
}

async fn cleanup(root: DbConn, db: DbConn, schema: String) -> anyhow::Result<()> {
    db.close().await?;
    anyhow::ensure!(schema.starts_with("checkout_payment_test_"));
    root.execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await?;
    root.close().await?;
    Ok(())
}

async fn fixtures(db: &DbConn) -> anyhow::Result<()> {
    db.execute_unprepared(
        r#"
INSERT INTO "user" (id, uuid, name, email, password, first_login, enabled, role, created_at, updated_at)
VALUES (1, gen_random_uuid(), 'Buyer', 'buyer@test.local', 'unused', false, true, 'Customer', now(), now());
INSERT INTO customer (id, uuid, user_id, name, email, cpf, marketing_consent, active, created_at, updated_at)
VALUES (1, gen_random_uuid(), 1, 'Buyer', 'buyer@test.local', '12345678901', false, true, now(), now());
INSERT INTO "user" (id, uuid, name, email, password, first_login, enabled, role, created_at, updated_at)
VALUES (2, gen_random_uuid(), 'Other Buyer', 'other-buyer@test.local', 'unused', false, true, 'Customer', now(), now());
INSERT INTO customer (id, uuid, user_id, name, email, cpf, marketing_consent, active, created_at, updated_at)
VALUES (2, gen_random_uuid(), 2, 'Other Buyer', 'other-buyer@test.local', '98765432100', false, true, now(), now());
INSERT INTO tenant (id, uuid, business_name, tax_id, postal_code, created_at, updated_at)
VALUES (1, gen_random_uuid(), 'Seller', '12345678000100', '01001000', now(), now());
INSERT INTO tenant (id, uuid, business_name, tax_id, postal_code, created_at, updated_at)
VALUES (2, gen_random_uuid(), 'Second Seller', '12345678000200', '01001000', now(), now());
INSERT INTO product (id, uuid, tenant_id, name, slug, active, ncm, origem_mercadoria, created_at, updated_at)
VALUES (1, gen_random_uuid(), 1, 'Product', 'product', true, '12345678', 0, now(), now());
INSERT INTO product (id, uuid, tenant_id, name, slug, active, ncm, origem_mercadoria, created_at, updated_at)
VALUES (2, gen_random_uuid(), 2, 'Second Product', 'second-product', true, '12345678', 0, now(), now());
INSERT INTO sku (id, uuid, tenant_id, product_id, code, variant_key, price_cents, active, weight_g, length_mm, width_mm, height_mm, created_at, updated_at)
VALUES (1, gen_random_uuid(), 1, 1, 'SKU-1', 'default', 5000, true, 500, 131, 91, 21, now(), now());
INSERT INTO sku (id, uuid, tenant_id, product_id, code, variant_key, price_cents, active, weight_g, length_mm, width_mm, height_mm, created_at, updated_at)
VALUES (2, gen_random_uuid(), 2, 2, 'SKU-2', 'default', 6500, true, 500, 131, 91, 21, now(), now());
INSERT INTO sku_stock (uuid, tenant_id, sku_id, quantity, reserved, created_at, updated_at)
VALUES (gen_random_uuid(), 1, 1, 10, 0, now(), now());
INSERT INTO sku_stock (uuid, tenant_id, sku_id, quantity, reserved, created_at, updated_at)
VALUES (gen_random_uuid(), 2, 2, 10, 0, now(), now());
INSERT INTO shipping_rate (uuid, tenant_id, uf, price_cents, created_at, updated_at)
VALUES (gen_random_uuid(), 1, 'SP', 1500, now(), now());
INSERT INTO shipping_rate (uuid, tenant_id, uf, price_cents, created_at, updated_at)
VALUES (gen_random_uuid(), 2, 'SP', 1800, now(), now());
"#,
    )
    .await?;
    Ok(())
}

async fn cart_fixture(
    db: &DbConn,
    tenant_id: Option<i64>,
    customer_id: i64,
    sku_ids: &[i64],
) -> anyhow::Result<i64> {
    let row = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO cart (uuid, tenant_id, customer_id, status, expires_at, created_at, updated_at) VALUES (gen_random_uuid(), $1, $2, 'active', now() + interval '15 minutes', now(), now()) RETURNING id",
            [tenant_id.into(), customer_id.into()],
        ))
        .await?
        .ok_or_else(|| anyhow::anyhow!("cart insert returned no row"))?;
    let cart_id: i64 = row.try_get("", "id")?;
    for sku_id in sku_ids {
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO cart_item (uuid, tenant_id, cart_id, sku_id, quantity, unit_price_cents, created_at, updated_at) VALUES (gen_random_uuid(), $1, $2, $3, 1, 5000, now(), now())",
            [tenant_id.into(), cart_id.into(), (*sku_id).into()],
        ))
        .await?;
    }
    Ok(cart_id)
}

async fn cart_counts(db: &DbConn, cart_id: i64) -> anyhow::Result<(i64, i64)> {
    let cart_count = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT COUNT(*) AS count FROM cart WHERE id=$1",
            [cart_id.into()],
        ))
        .await?
        .ok_or_else(|| anyhow::anyhow!("cart count query returned no row"))?
        .try_get("", "count")?;
    let item_count = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT COUNT(*) AS count FROM cart_item WHERE cart_id=$1",
            [cart_id.into()],
        ))
        .await?
        .ok_or_else(|| anyhow::anyhow!("cart item count query returned no row"))?
        .try_get("", "count")?;
    Ok((cart_count, item_count))
}

fn user() -> User {
    User {
        id: Some(1),
        uuid: None,
        email: "buyer@test.local".into(),
        name: Some("Buyer".into()),
        password: String::new(),
        enabled: true,
        first_login: false,
        tenant_id: None,
        role: Role::Customer,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
    }
}

fn user_with_role(id: i64, role: Role, tenant_id: Option<i64>) -> User {
    let mut user = user();
    user.id = Some(id);
    user.email = format!("user-{id}@test.local");
    user.role = role;
    user.tenant_id = tenant_id;
    user
}

fn state(db: DbConn) -> AppState {
    let shipping_keys = Arc::new(ShippingKeyRing::for_test());
    let shipping =
        Correios::with_base_url_for_test(db.clone(), shipping_keys.clone(), "http://127.0.0.1:1")
            .expect("test Correios provider configuration");
    let s3 = aws_sdk_s3::Client::from_conf(
        aws_sdk_s3::Config::builder()
            .behavior_version(aws_config::BehaviorVersion::latest())
            .build(),
    );
    AppState {
        conn: Arc::new(db),
        storage: Arc::new(business::gateway::storage_gateway::StorageGateway::new(
            "unused-test-bucket".into(),
            "http://localhost.invalid".into(),
            s3,
        )),
        shipping: Arc::new(shipping),
        shipping_keys,
        payment_keys: Arc::new(PaymentKeyRing::default()),
        gateway_token: None,
    }
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/checkout/quotes", post(quote))
        .route(
            "/checkout/quotes/{id}/shipping-selection",
            post(select_shipping),
        )
        .route("/purchases", post(create_purchase))
        .route("/purchases/{id}", get(get_purchase))
        .route("/purchases/{id}/payments", get(payments))
        .route("/purchases/{id}/payments/submit", post(submit))
        .route("/purchases/{id}/payments/status", get(status))
        .route("/orders/{id}", get(get_by_id))
        .route("/orders/{id}/status-history", get(history))
        .route("/payments/{id}/transactions", get(transactions))
        .with_state(state)
}

async fn request(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
    idempotency_key: Option<&str>,
) -> anyhow::Result<(StatusCode, Value)> {
    request_as(app, method, uri, body, idempotency_key, user()).await
}

async fn request_as(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
    idempotency_key: Option<&str>,
    actor: User,
) -> anyhow::Result<(StatusCode, Value)> {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    if let Some(key) = idempotency_key {
        builder = builder.header("Idempotency-Key", key);
    }
    let mut request = builder.body(Body::from(
        body.map(|value| value.to_string()).unwrap_or_default(),
    ))?;
    request.extensions_mut().insert(actor);
    request.extensions_mut().insert(Locale::En);
    request.extensions_mut().insert(TenantContext::Marketplace);
    let response = app.oneshot(request).await?;
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await?;
    let body = serde_json::from_slice(&bytes)?;
    Ok((status, body))
}

async fn create_purchase_from_quote(
    app: Router,
    idempotency_key: &str,
    sku_ids: &[i64],
) -> anyhow::Result<i64> {
    let items: Vec<Value> = sku_ids
        .iter()
        .map(|sku_id| json!({"skuId":sku_id,"quantity":1}))
        .collect();
    let (status, quote) = request(
        app.clone(),
        Method::POST,
        "/checkout/quotes",
        Some(json!({
            "items":items,
            "addressId":null,
            "shippingAddress":{
                "recipient":"Buyer",
                "addressLine1":"Street 1",
                "addressLine2":null,
                "locality":"Sao Paulo",
                "administrativeArea":"SP",
                "postalCode":"01001000",
                "countryCode":"BR"
            },
            "coupons":[]
        })),
        None,
    )
    .await?;
    anyhow::ensure!(
        status == StatusCode::CREATED,
        "quote returned {status}: {quote}"
    );
    let quote_id = quote["id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("quote response is missing id: {quote}"))?;
    let selections: Vec<Value> = quote["sellers"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("quote response is missing sellers: {quote}"))?
        .iter()
        .map(|seller| {
            json!({
                "tenantId": seller["tenantId"],
                "optionId": seller["shipping"]["selectedOption"]["id"]
            })
        })
        .collect();
    let (status, selected_quote) = request(
        app.clone(),
        Method::POST,
        &format!("/checkout/quotes/{quote_id}/shipping-selection"),
        Some(json!(selections)),
        None,
    )
    .await?;
    anyhow::ensure!(
        status == StatusCode::CREATED,
        "shipping selection returned {status}: {selected_quote}"
    );
    let quote_id = selected_quote["id"].as_i64().ok_or_else(|| {
        anyhow::anyhow!("selected quote response is missing id: {selected_quote}")
    })?;

    let (status, purchase) = request(
        app,
        Method::POST,
        "/purchases",
        Some(json!({
            "quoteId": quote_id,
            "email": "buyer@test.local",
            "phone": "11999999999"
        })),
        Some(idempotency_key),
    )
    .await?;
    anyhow::ensure!(
        status == StatusCode::CREATED,
        "purchase returned {status}: {purchase}"
    );
    purchase["id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("purchase response is missing id: {purchase}"))
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL and PAYMENT_PROVIDER_MODE=mock"]
async fn checkout_payment_http_enforces_ownership_and_mock_lifecycle() {
    assert_eq!(
        std::env::var("PAYMENT_PROVIDER_MODE").as_deref().ok(),
        Some("mock"),
        "run this test with PAYMENT_PROVIDER_MODE=mock"
    );
    assert_eq!(
        std::env::var("PAYMENT_MOCK_PROVIDER").as_deref().ok(),
        Some("mercado_pago"),
        "run this test with PAYMENT_MOCK_PROVIDER=mercado_pago"
    );

    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let app = router(state(db.clone()));

    let captured_cart_id = cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let captured_id = create_purchase_from_quote(app.clone(), "mock-capture", &[1])
        .await
        .unwrap();
    let (status, captured) = request(
        app.clone(),
        Method::POST,
        &format!("/purchases/{captured_id}/payments/submit"),
        Some(json!({
            "token":"mock:approved",
            "paymentMethodId":"visa",
            "installments":1
        })),
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{captured}");
    assert_eq!(captured["status"], "captured");
    assert_eq!(cart_counts(&db, captured_cart_id).await.unwrap(), (0, 0));
    assert!(
        captured["reference"]
            .as_str()
            .unwrap_or_default()
            .starts_with("mock:mercado_pago:captured:")
    );

    let (status, purchase_detail) = request(
        app.clone(),
        Method::GET,
        &format!("/purchases/{captured_id}"),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{purchase_detail}");
    assert_eq!(purchase_detail["status"], "paid", "{purchase_detail}");
    let order_id = purchase_detail["orders"][0]["id"].as_i64().unwrap();
    let payment_id = purchase_detail["payments"][0]["id"].as_i64().unwrap();

    let (status, customer_order) = request(
        app.clone(),
        Method::GET,
        &format!("/orders/{order_id}"),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{customer_order}");
    assert_eq!(customer_order["customerId"], 1);
    let (status, customer_history) = request(
        app.clone(),
        Method::GET,
        &format!("/orders/{order_id}/status-history"),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{customer_history}");
    assert!(
        customer_history
            .as_array()
            .is_some_and(|events| events.iter().any(|event| event["toStatus"] == "paid"))
    );

    let (status, payment_list) = request(
        app.clone(),
        Method::GET,
        &format!("/purchases/{captured_id}/payments"),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{payment_list}");
    assert_eq!(payment_list[0]["status"], "captured");

    let admin = user_with_role(30, Role::SysAdmin, None);
    for uri in [
        format!("/purchases/{captured_id}"),
        format!("/purchases/{captured_id}/payments"),
        format!("/orders/{order_id}"),
        format!("/orders/{order_id}/status-history"),
        format!("/payments/{payment_id}/transactions"),
    ] {
        let (status, response) =
            request_as(app.clone(), Method::GET, &uri, None, None, admin.clone())
                .await
                .unwrap();
        assert_eq!(status, StatusCode::OK, "SysAdmin GET {uri}: {response}");
    }

    for role in [Role::TenantOwner, Role::TenantUser] {
        let actor = user_with_role(20, role.clone(), Some(1));
        let (status, purchase) = request_as(
            app.clone(),
            Method::GET,
            &format!("/purchases/{captured_id}"),
            None,
            None,
            actor.clone(),
        )
        .await
        .unwrap();
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "{role:?} purchase: {purchase}"
        );

        let (status, payments) = request_as(
            app.clone(),
            Method::GET,
            &format!("/purchases/{captured_id}/payments"),
            None,
            None,
            user_with_role(20, role.clone(), Some(1)),
        )
        .await
        .unwrap();
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "{role:?} payments: {payments}"
        );

        let (status, order) = request_as(
            app.clone(),
            Method::GET,
            &format!("/orders/{order_id}"),
            None,
            None,
            actor,
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::OK, "{role:?} order: {order}");
        assert_eq!(order["tenantId"], 1);

        let (status, order_history) = request_as(
            app.clone(),
            Method::GET,
            &format!("/orders/{order_id}/status-history"),
            None,
            None,
            user_with_role(20, role.clone(), Some(1)),
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::OK, "{role:?} history: {order_history}");
        assert!(
            order_history
                .as_array()
                .is_some_and(|events| events.iter().any(|event| event["toStatus"] == "paid")),
            "{role:?} history does not contain the paid event: {order_history}"
        );

        let (status, transactions) = request_as(
            app.clone(),
            Method::GET,
            &format!("/payments/{payment_id}/transactions"),
            None,
            None,
            user_with_role(20, role.clone(), Some(1)),
        )
        .await
        .unwrap();
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "{role:?} transaction access: {transactions}"
        );
    }

    let (status, transactions) = request_as(
        app.clone(),
        Method::GET,
        &format!("/payments/{payment_id}/transactions"),
        None,
        None,
        user_with_role(2, Role::Customer, None),
    )
    .await
    .unwrap();
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "foreign Customer transaction access: {transactions}"
    );

    let (status, payment_transactions) = request(
        app.clone(),
        Method::GET,
        &format!("/payments/{payment_id}/transactions"),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{payment_transactions}");
    assert_eq!(payment_transactions[0]["status"], "captured");
    assert_eq!(payment_transactions[0]["gatewayProvider"], "mercado_pago");

    let (status, purchase_row) = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM purchase WHERE id=$1",
            [captured_id.into()],
        ))
        .await
        .unwrap()
        .map(|row| (StatusCode::OK, row.try_get::<String>("", "status").unwrap()))
        .unwrap_or((StatusCode::NOT_FOUND, String::new()));
    assert_eq!(status, StatusCode::OK);
    assert_eq!(purchase_row, "paid");

    let pending_cart_id = cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let pending_id = create_purchase_from_quote(app.clone(), "mock-pending", &[1])
        .await
        .unwrap();
    let (status, pending) = request(
        app.clone(),
        Method::POST,
        &format!("/purchases/{pending_id}/payments/submit"),
        Some(json!({
            "token":"mock:pending",
            "paymentMethodId":"visa",
            "installments":1
        })),
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{pending}");
    assert_eq!(pending["status"], "pending");
    assert_eq!(cart_counts(&db, pending_cart_id).await.unwrap(), (1, 1));

    for (index, (role, tenant_id)) in [
        (Role::SysAdmin, None),
        (Role::TenantOwner, Some(1)),
        (Role::TenantUser, Some(1)),
    ]
    .into_iter()
    .enumerate()
    {
        let actor = user_with_role(10 + index as i64, role.clone(), tenant_id);
        let (status, body) = request_as(
            app.clone(),
            Method::POST,
            &format!("/purchases/{pending_id}/payments/submit"),
            Some(json!({
                "token":"mock:approved",
                "paymentMethodId":"visa",
                "installments":1
            })),
            None,
            actor,
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::FORBIDDEN, "{role:?} submit: {body}");

        let (status, body) = request_as(
            app.clone(),
            Method::GET,
            &format!("/purchases/{pending_id}/payments/status"),
            None,
            None,
            user_with_role(10 + index as i64, role.clone(), tenant_id),
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::FORBIDDEN, "{role:?} status: {body}");
    }

    for (method, uri, body) in [
        (
            Method::POST,
            format!("/purchases/{pending_id}/payments/submit"),
            Some(json!({
                "token":"mock:approved",
                "paymentMethodId":"visa",
                "installments":1
            })),
        ),
        (
            Method::GET,
            format!("/purchases/{pending_id}/payments/status"),
            None,
        ),
    ] {
        let (status, response) = request_as(
            app.clone(),
            method,
            &uri,
            body,
            None,
            user_with_role(2, Role::Customer, None),
        )
        .await
        .unwrap();
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "foreign Customer: {response}"
        );
    }

    let (status, retrieved) = request(
        app.clone(),
        Method::GET,
        &format!("/purchases/{pending_id}/payments/status"),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{retrieved}");
    assert_eq!(retrieved["status"], "pending");
    assert_eq!(retrieved["reference"], pending["reference"]);

    let marketplace_cart = cart_fixture(&db, None, 1, &[1, 2]).await.unwrap();
    let seller_one_cart = cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let seller_two_cart = cart_fixture(&db, Some(2), 1, &[2]).await.unwrap();
    let other_customer_cart = cart_fixture(&db, None, 2, &[1, 2]).await.unwrap();
    let marketplace_purchase_id =
        create_purchase_from_quote(app.clone(), "mock-marketplace-capture", &[1, 2])
            .await
            .unwrap();
    let (status, marketplace_capture) = request(
        app,
        Method::POST,
        &format!("/purchases/{marketplace_purchase_id}/payments/submit"),
        Some(json!({
            "token":"mock:approved",
            "paymentMethodId":"visa",
            "installments":1
        })),
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{marketplace_capture}");
    assert_eq!(marketplace_capture["status"], "captured");
    assert_eq!(cart_counts(&db, marketplace_cart).await.unwrap(), (0, 0));
    assert_eq!(cart_counts(&db, seller_one_cart).await.unwrap(), (1, 1));
    assert_eq!(cart_counts(&db, seller_two_cart).await.unwrap(), (1, 1));
    assert_eq!(cart_counts(&db, other_customer_cart).await.unwrap(), (1, 2));

    cleanup(root, db, schema).await.unwrap();
}

async fn request_in(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
    context: TenantContext,
) -> anyhow::Result<StatusCode> {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    let mut request = builder.body(Body::from(
        body.map(|value| value.to_string()).unwrap_or_default(),
    ))?;
    request.extensions_mut().insert(user());
    request.extensions_mut().insert(Locale::En);
    request.extensions_mut().insert(context);
    Ok(app.oneshot(request).await?.status())
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL and PAYMENT_PROVIDER_MODE=mock"]
async fn tenant_bound_purchase_is_not_payable_or_readable_from_another_tenant() {
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let app = router(state(db.clone()));
    let id = create_purchase_from_quote(app.clone(), "tenant-bound", &[1])
        .await
        .unwrap();
    db.execute_unprepared(&format!("UPDATE purchase SET tenant_id=1 WHERE id={id}"))
        .await
        .unwrap();
    let pay = Some(json!({"token":"mock:approved","paymentMethodId":"visa","installments":1}));

    // SR-MKT-009: another tenant's context sees nothing and charges nothing.
    let foreign = TenantContext::Fixed(2);
    for (method, path, body) in [
        (Method::POST, "payments/submit", pay.clone()),
        (Method::GET, "payments/status", None),
        (Method::GET, "payments", None),
        (Method::GET, "", None),
    ] {
        let uri = format!("/purchases/{id}/{path}").trim_end_matches('/').to_string();
        let status = request_in(app.clone(), method, &uri, body, foreign)
            .await
            .unwrap();
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
    }
    let charged = db
        .query_one_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*)::bigint AS n FROM payment WHERE status='captured'".to_string(),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap();
    assert_eq!(charged, 0);

    // The owning tenant context and the marketplace context still reach it.
    for context in [TenantContext::Fixed(1), TenantContext::Marketplace] {
        let status = request_in(
            app.clone(),
            Method::GET,
            &format!("/purchases/{id}/payments"),
            None,
            context,
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::OK);
    }
    let status = request_in(
        app,
        Method::POST,
        &format!("/purchases/{id}/payments/submit"),
        pay,
        TenantContext::Fixed(1),
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK);
    cleanup(root, db, schema).await.unwrap();
}

fn quote_body(sku_ids: &[i64]) -> Value {
    let items: Vec<Value> = sku_ids
        .iter()
        .map(|sku_id| json!({"skuId":sku_id,"quantity":1}))
        .collect();
    json!({
        "items":items,
        "addressId":null,
        "shippingAddress":{
            "recipient":"Buyer","addressLine1":"Street 1","addressLine2":null,
            "locality":"Sao Paulo","administrativeArea":"SP","postalCode":"01001000","countryCode":"BR"
        },
        "coupons":[]
    })
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL"]
async fn marketplace_quote_selection_and_purchase_per_seller() {
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    db.execute_unprepared("UPDATE tenant SET listed=false WHERE id=2")
        .await
        .unwrap();
    let app = router(state(db.clone()));

    // TC-13: a seller with the listing flag off is bought like any other in marketplace mode.
    create_purchase_from_quote(app.clone(), "unlisted", &[2])
        .await
        .unwrap();

    // TC-02: one block per seller; the quote totals are the sums.
    let (status, quote) = request(
        app.clone(),
        Method::POST,
        "/checkout/quotes",
        Some(quote_body(&[1, 2])),
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::CREATED, "{quote}");
    let sellers = quote["sellers"].as_array().unwrap();
    assert_eq!(sellers.len(), 2);
    assert_eq!(
        (sellers[0]["tenantId"].as_i64(), sellers[0]["subtotalCents"].as_i64(), sellers[0]["shippingCents"].as_i64()),
        (Some(1), Some(5000), Some(1500))
    );
    assert_eq!(
        (sellers[1]["tenantId"].as_i64(), sellers[1]["subtotalCents"].as_i64(), sellers[1]["shippingCents"].as_i64()),
        (Some(2), Some(6500), Some(1800))
    );
    let sum: i64 = sellers.iter().map(|s| s["totalCents"].as_i64().unwrap()).sum();
    assert_eq!(quote["totalCents"].as_i64().unwrap(), sum);

    // TC-16: a selection must choose for every seller.
    let quote_id = quote["id"].as_i64().unwrap();
    let option = |seller: &Value| seller["shipping"]["selectedOption"]["id"].clone();
    let (status, body) = request(
        app.clone(),
        Method::POST,
        &format!("/checkout/quotes/{quote_id}/shipping-selection"),
        Some(json!([{"tenantId":1,"optionId":option(&sellers[0])}])),
        None,
    )
    .await
    .unwrap();
    assert!(status.is_client_error(), "{status}: {body}");

    // TC-16 step 1 and TC-05: purchase straight from the fresh quote uses the default options.
    let (status, purchase) = request(
        app.clone(),
        Method::POST,
        "/purchases",
        Some(json!({"quoteId":quote_id,"email":"buyer@test.local","phone":"11999999999"})),
        Some("default-selection"),
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::CREATED, "{purchase}");
    let orders = purchase["orders"].as_array().unwrap();
    assert_eq!(orders.len(), 2);
    let mut tenants: Vec<_> = orders.iter().map(|o| o["tenantId"].as_i64().unwrap()).collect();
    tenants.sort();
    assert_eq!(tenants, [1, 2]);
    let shipping: Vec<_> = orders.iter().map(|o| o["shippingCents"].as_i64().unwrap()).collect();
    assert!(shipping.contains(&1500) && shipping.contains(&1800), "{shipping:?}");
    let order_total: i64 = orders.iter().map(|o| o["totalCents"].as_i64().unwrap()).sum();
    assert_eq!(purchase["totalCents"].as_i64().unwrap(), order_total);

    // TC-01: a tenant-less cart holds items of several sellers.
    let cart_id = cart_fixture(&db, None, 1, &[1, 2]).await.unwrap();
    assert_eq!(cart_counts(&db, cart_id).await.unwrap(), (1, 2));
    cleanup(root, db, schema).await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL and PAYMENT_PROVIDER_MODE=mock"]
async fn reconciliation_backs_off_and_cancels_payments_past_the_window() {
    // NOV-5 TC-11/23/24: SR-PAY-021, SR-PAY-023
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let state = state(db.clone());
    let app = router(state.clone());

    cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let purchase_id = create_purchase_from_quote(app.clone(), "recon-aged", &[1])
        .await
        .unwrap();
    let (status, submitted) = request(
        app.clone(),
        Method::POST,
        &format!("/purchases/{purchase_id}/payments/submit"),
        Some(json!({"token":"mock:pending","paymentMethodId":"visa","installments":1})),
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{submitted}");
    assert_eq!(submitted["status"], "pending");

    // Fresh payment: backoff keeps it out of the claim.
    let claimed = crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
        .await
        .unwrap();
    assert_eq!(claimed, 0);

    // Past the 3h window: cancelled at the provider, purchase fails, no refund.
    db.execute_unprepared(
        "UPDATE payment SET created_at = now() - interval '4 hours', updated_at = now() - interval '4 hours'",
    )
    .await
    .unwrap();
    let handled = crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
        .await
        .unwrap();
    assert_eq!(handled, 1);
    let (status, purchase) = request(
        app.clone(),
        Method::GET,
        &format!("/purchases/{purchase_id}"),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{purchase}");
    assert_eq!(purchase["payments"][0]["status"], "failed", "{purchase}");
    assert_eq!(purchase["status"], "payment_failed", "{purchase}");

    // Resolved payments are not claimed again.
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        0
    );
    cleanup(root, db, schema).await.unwrap();
}
