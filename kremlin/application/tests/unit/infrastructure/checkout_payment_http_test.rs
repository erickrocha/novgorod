use crate::{
    AppState,
    commons::{i18n::Locale, tenant_context::TenantContext},
    endpoints::{
        checkout_payment_endpoint::{pagseguro_webhook, status, submit, webhook},
        checkout_quote_endpoint::{quote, select_shipping},
        orders_endpoint::{
            create_purchase, get_by_id, get_purchase, history, payments, transactions,
        },
    },
    infrastructure::{
        correios::Correios,
        payment_credentials::{
            MercadoPagoCredentials, PagSeguroCredentials, PaymentKeyRing,
            TenantCredentialsPayload,
        },
        shipping_credentials::ShippingKeyRing,
    },
};
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::Path,
    http::{HeaderMap, Method, Request, StatusCode},
    routing::{get, post},
};
use business::{
    domain::{enums::Role, user::User},
    sea_orm::{ConnectOptions, ConnectionTrait, Database, DbBackend, DbConn, Statement, TransactionTrait},
};
use migration::{Migrator, MigratorTrait};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
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

async fn assert_payment_markers_absent(db: &DbConn, markers: &[&str]) {
    for table in [
        "payment",
        "payment_transaction",
        "credit_card_details",
        "payment_allocation",
    ] {
        for marker in markers {
            let row = db
                .query_one_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    format!(
                        "SELECT count(*) AS count FROM {table} WHERE CAST({table} AS TEXT) ILIKE $1"
                    ),
                    [format!("%{marker}%").into()],
                ))
                .await
                .unwrap()
                .unwrap();
            let count: i64 = row.try_get("", "count").unwrap();
            assert_eq!(count, 0, "marker {marker} found in {table}");
        }
    }
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
        .route("/webhooks/pagseguro", post(pagseguro_webhook))
        .route("/webhooks/mercadopago", post(webhook))
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

async fn webhook_request(
    app: Router,
    body: &[u8],
    signature: &str,
) -> anyhow::Result<StatusCode> {
    webhook_request_with_headers(app, body, &[signature], None).await
}

async fn webhook_request_with_provider_base(
    app: Router,
    body: &[u8],
    signature: &str,
    provider_base_url: Option<&str>,
) -> anyhow::Result<StatusCode> {
    webhook_request_with_headers(app, body, &[signature], provider_base_url).await
}

async fn webhook_request_with_headers(
    app: Router,
    body: &[u8],
    signatures: &[&str],
    provider_base_url: Option<&str>,
) -> anyhow::Result<StatusCode> {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri("/webhooks/pagseguro")
        .header("content-type", "application/json");
    for signature in signatures {
        builder = builder.header("x-authenticity-token", *signature);
    }
    if let Some(base_url) = provider_base_url {
        builder = builder.header("x-test-pagseguro-base-url", base_url);
    }
    let request = builder.body(Body::from(body.to_vec()))?;
    Ok(app.oneshot(request).await?.status())
}

async fn mercado_pago_webhook_request(
    app: Router,
    body: &[u8],
    request_id: &str,
    signature: &str,
) -> anyhow::Result<StatusCode> {
    let request = Request::builder()
        .method(Method::POST)
        .uri("/webhooks/mercadopago")
        .header("content-type", "application/json")
        .header("x-request-id", request_id)
        .header("x-signature", signature)
        .body(Body::from(body.to_vec()))?;
    Ok(app.oneshot(request).await?.status())
}

fn pagbank_signature(token: &str, body: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(token.as_bytes());
    digest.update(b"-");
    digest.update(body);
    format!("{:x}", digest.finalize())
}

struct EnvironmentRestore(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl EnvironmentRestore {
    fn capture(names: &[&'static str]) -> Self {
        Self(
            names
                .iter()
                .map(|name| (*name, std::env::var_os(name)))
                .collect(),
        )
    }
}

impl Drop for EnvironmentRestore {
    fn drop(&mut self) {
        for (name, value) in self.0.drain(..) {
            unsafe {
                if let Some(value) = value {
                    std::env::set_var(name, value);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }
}

fn mercado_pago_signature(secret: &str, timestamp: &str, request_id: &str, data_id: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let manifest = format!("id:{data_id};request-id:{request_id};ts:{timestamp};");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(manifest.as_bytes());
    let digest = mac.finalize().into_bytes();
    let hex = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    format!("ts={timestamp},v1={hex}")
}

#[derive(Clone)]
struct PagBankDoubleState {
    authorization: Arc<std::sync::Mutex<Option<String>>>,
    amount_cents: i64,
}

async fn pagbank_status_double(
    axum::extract::State(state): axum::extract::State<PagBankDoubleState>,
    Path(reference): Path<String>,
    headers: HeaderMap,
) -> axum::Json<Value> {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .map(str::to_owned);
    let external_reference = reference.splitn(6, ':').nth(5).unwrap_or_default();
    axum::Json(json!({
        "id": reference,
        "reference_id": external_reference,
        "status": "DECLINED",
        "amount": {"value": state.amount_cents, "currency": "BRL"}
    }))
}

#[derive(Clone)]
struct MercadoPagoDoubleState {
    authorization: Arc<std::sync::Mutex<Vec<String>>>,
    amount_cents: i64,
    external_reference: String,
    collector_id: i64,
    status: String,
}

async fn mercado_pago_status_double(
    axum::extract::State(state): axum::extract::State<MercadoPagoDoubleState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> axum::Json<Value> {
    state.authorization.lock().unwrap().push(
        headers
            .get("authorization")
            .and_then(|header| header.to_str().ok())
            .unwrap_or_default()
            .to_owned(),
    );
    axum::Json(json!({
        "id": id,
        "status": state.status,
        "transaction_amount": state.amount_cents as f64 / 100.0,
        "currency_id": "BRL",
        "external_reference": state.external_reference,
        "collector_id": state.collector_id,
        "payment_type_id": "credit_card",
        "payment_method_id": "visa"
    }))
}

async fn mercado_pago_cancel_double(
    axum::extract::State(state): axum::extract::State<MercadoPagoDoubleState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> axum::Json<Value> {
    state.authorization.lock().unwrap().push(
        headers
            .get("authorization")
            .and_then(|header| header.to_str().ok())
            .unwrap_or_default()
            .to_owned(),
    );
    axum::Json(json!({
        "id": id,
        "status": "cancelled",
        "transaction_amount": state.amount_cents as f64 / 100.0,
        "currency_id": "BRL",
        "external_reference": state.external_reference,
        "collector_id": state.collector_id,
        "payment_type_id": "credit_card",
        "payment_method_id": "visa"
    }))
}

async fn tc28_provider_error_response() -> (StatusCode, axum::Json<Value>) {
    (
        StatusCode::BAD_GATEWAY,
        axum::Json(json!({
            "message": "token=tc28-token-body-marker tax_id=tc28-tax-body-marker card=tc28-card-body-marker"
        })),
    )
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

#[tokio::test]
#[ignore = "requires isolated PostgreSQL, PagBank test keyring, and PAYMENT_PROVIDER_MODE=mock"]
async fn pagseguro_webhook_authenticates_then_refreshes_through_mock_provider() {
    const TOKEN: &str = "pagbank-account-token-for-tests-only";
    assert_eq!(
        std::env::var("PAYMENT_PROVIDER_MODE").as_deref().ok(),
        Some("mock")
    );
    assert_eq!(
        std::env::var("PAYMENT_MOCK_PROVIDER").as_deref().ok(),
        Some("pagseguro")
    );

    let payment_keys = Arc::new(PaymentKeyRing::from_env().unwrap());
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let credentials = TenantCredentialsPayload::PagSeguro(PagSeguroCredentials {
        token: TOKEN.into(),
        public_key: None,
        environment: Some("sandbox".into()),
    });
    let (key_version, encrypted) = payment_keys.encrypt_payload(1, &credentials).unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO tenant_payment_settings (tenant_id, provider, credentials, key_version) VALUES ($1, 'pagseguro', $2, $3)",
        [1_i64.into(), encrypted.into(), key_version.into()],
    ))
    .await
    .unwrap();
    let mut app_state = state(db.clone());
    app_state.payment_keys = payment_keys.clone();
    let app = router(app_state);

    cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let purchase_id = create_purchase_from_quote(app.clone(), "pagbank-mock", &[1])
        .await
        .unwrap();
    let (status, submitted) = request(
        app.clone(),
        Method::POST,
        &format!("/purchases/{purchase_id}/payments/submit"),
        Some(json!({
            "token":"mock:pending",
            "paymentMethodId":"visa",
            "installments":1
        })),
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{submitted}");
    assert_eq!(submitted["status"], "pending");

    let payment = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT attempt_key FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let attempt_key: String = payment.try_get("", "attempt_key").unwrap();
    let reference = format!("torg-{purchase_id}-{attempt_key}");
    let amount_cents: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT amount_cents FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "amount_cents")
        .unwrap();
    let body = format!(r#"{{"reference_id":"{reference}","status":"PAID"}}"#);
    let valid_signature = pagbank_signature(TOKEN, body.as_bytes());
    let authorization = Arc::new(std::sync::Mutex::new(None));
    let double = Router::new()
        .route("/charges/{reference}", get(pagbank_status_double))
        .with_state(PagBankDoubleState {
            authorization: authorization.clone(),
            amount_cents,
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, double).await.unwrap() });
    let provider_base_url = format!("http://{address}");

    for signatures in [vec![], vec![valid_signature.as_str(), valid_signature.as_str()], vec!["not-a-digest"]] {
        let rejected = webhook_request_with_headers(
            app.clone(),
            body.as_bytes(),
            &signatures,
            Some(&provider_base_url),
        )
        .await
        .unwrap();
        assert_eq!(rejected, StatusCode::UNAUTHORIZED);
    }
    let rejected = webhook_request_with_provider_base(
        app.clone(),
        body.as_bytes(),
        &"0".repeat(64),
        Some(&provider_base_url),
    )
    .await
    .unwrap();
    assert_eq!(rejected, StatusCode::UNAUTHORIZED);
    let wrong_tenant_signature = pagbank_signature("other-tenant-token", body.as_bytes());
    let rejected = webhook_request_with_provider_base(
        app.clone(),
        body.as_bytes(),
        &wrong_tenant_signature,
        Some(&provider_base_url),
    )
        .await
        .unwrap();
    assert_eq!(rejected, StatusCode::UNAUTHORIZED);
    let tampered_body = body.replace("PAID", "PENDING");
    let rejected = webhook_request_with_provider_base(
        app.clone(),
        tampered_body.as_bytes(),
        &valid_signature,
        Some(&provider_base_url),
    )
        .await
        .unwrap();
    assert_eq!(rejected, StatusCode::UNAUTHORIZED);
    assert_eq!(authorization.lock().unwrap().as_deref(), None);

    let payment_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(payment_status, "pending");

    let accepted = webhook_request_with_provider_base(
        app.clone(),
        body.as_bytes(),
        &valid_signature,
        Some(&provider_base_url),
    )
        .await
        .unwrap();
    assert_eq!(accepted, StatusCode::OK);
    assert_eq!(
        authorization.lock().unwrap().as_deref(),
        Some("Bearer pagbank-account-token-for-tests-only")
    );

    let payment_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(payment_status, "failed");
    server.abort();

    let unknown_reference = format!(
        r#"{{"reference_id":"torg-{purchase_id}-unknown-attempt","status":"PAID"}}"#
    );
    let unknown_signature = pagbank_signature(TOKEN, unknown_reference.as_bytes());
    crate::test_log_capture::install();
    crate::test_log_capture::clear();
    let unknown_status = webhook_request(app.clone(), unknown_reference.as_bytes(), &unknown_signature)
        .await
        .unwrap();
    assert_eq!(unknown_status, StatusCode::OK);
    let captured_logs = crate::test_log_capture::contents();
    assert!(captured_logs.contains("WARN pagseguro webhook: unknown payment attempt"));
    assert!(!captured_logs.contains(&unknown_reference));
    assert!(!captured_logs.contains(TOKEN));
    let unchanged_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(unchanged_status, "failed");
    cleanup(root, db, schema).await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL, test keyring, and PAYMENT_PROVIDER_MODE=mock"]
async fn mercado_pago_webhook_uses_tenant_secret_and_handles_repeated_ids() {
    const TENANT_TOKEN: &str = "tenant-mp-token-for-tests-only";
    const TENANT_SECRET: &str = "tenant-mp-secret-for-tests-only";
    const OTHER_TENANT_SECRET: &str = "other-tenant-mp-secret";
    const PLATFORM_SECRET: &str = "platform-mp-secret";
    const PAYMENT_ID: &str = "987654321";
    const REQUEST_ID: &str = "mp-request-id-for-tests";
    const TIMESTAMP: &str = "1727280000";

    assert_eq!(
        std::env::var("PAYMENT_PROVIDER_MODE").as_deref().ok(),
        Some("mock")
    );
    let _environment_restore = EnvironmentRestore::capture(&["MP_API_BASE_URL", "MP_WEBHOOK_SECRET"]);
    let payment_keys = Arc::new(PaymentKeyRing::from_env().unwrap());
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let credentials = TenantCredentialsPayload::MercadoPago(MercadoPagoCredentials {
        access_token: TENANT_TOKEN.into(),
        public_key: Some("tenant-mp-public-key".into()),
        collector_id: Some(4567),
        webhook_secret: Some(TENANT_SECRET.into()),
    });
    let (key_version, encrypted) = payment_keys.encrypt_payload(1, &credentials).unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO tenant_payment_settings (tenant_id, provider, credentials, key_version) VALUES ($1, 'mercado_pago', $2, $3)",
        [1_i64.into(), encrypted.into(), key_version.into()],
    ))
    .await
    .unwrap();
    let mut app_state = state(db.clone());
    app_state.payment_keys = payment_keys.clone();
    let app = router(app_state);

    cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let purchase_id = create_purchase_from_quote(app.clone(), "mp-webhook-mock", &[1])
        .await
        .unwrap();
    let (status, submitted) = request(
        app.clone(),
        Method::POST,
        &format!("/purchases/{purchase_id}/payments/submit"),
        Some(json!({
            "token":"mock:pending",
            "paymentMethodId":"visa",
            "installments":1
        })),
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{submitted}");
    assert_eq!(submitted["status"], "pending");
    let payment = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT amount_cents,attempt_key FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let amount_cents: i64 = payment.try_get("", "amount_cents").unwrap();
    let attempt_key: String = payment.try_get("", "attempt_key").unwrap();
    let external_reference = format!("torg-{purchase_id}-{attempt_key}");
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET gateway_provider='mercado_pago',gateway_reference=$1 WHERE purchase_id=$2",
        [PAYMENT_ID.into(), purchase_id.into()],
    ))
    .await
    .unwrap();

    let authorization = Arc::new(std::sync::Mutex::new(Vec::new()));
    let double = Router::new()
        .route("/v1/payments/{id}", get(mercado_pago_status_double))
        .with_state(MercadoPagoDoubleState {
            authorization: authorization.clone(),
            amount_cents,
            external_reference,
            collector_id: 4567,
            status: "rejected".into(),
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, double).await.unwrap() });
    unsafe {
        std::env::set_var("MP_API_BASE_URL", format!("http://{address}"));
        std::env::set_var("MP_WEBHOOK_SECRET", PLATFORM_SECRET);
    }

    let body = format!(r#"{{"data":{{"id":"{PAYMENT_ID}"}}}}"#);
    for secret in [PLATFORM_SECRET, OTHER_TENANT_SECRET] {
        let signature = mercado_pago_signature(secret, TIMESTAMP, REQUEST_ID, PAYMENT_ID);
        let rejected = mercado_pago_webhook_request(
            app.clone(),
            body.as_bytes(),
            REQUEST_ID,
            &signature,
        )
        .await
        .unwrap();
        assert_eq!(rejected, StatusCode::UNAUTHORIZED);
    }
    assert!(authorization.lock().unwrap().is_empty());
    let pending_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(pending_status, "pending");

    let valid_signature = mercado_pago_signature(TENANT_SECRET, TIMESTAMP, REQUEST_ID, PAYMENT_ID);
    let accepted = mercado_pago_webhook_request(
        app.clone(),
        body.as_bytes(),
        REQUEST_ID,
        &valid_signature,
    )
    .await
    .unwrap();
    assert_eq!(accepted, StatusCode::OK);
    assert_eq!(
        authorization.lock().unwrap().as_slice(),
        &[format!("Bearer {TENANT_TOKEN}")]
    );

    for index in 0..10_000 {
        let data_id = (1_000_000_000_i64 + index).to_string();
        let request_id = format!("tc22-request-{index}");
        let body = format!(r#"{{"data":{{"id":{data_id}}}}}"#);
        let signature = mercado_pago_signature(TENANT_SECRET, TIMESTAMP, &request_id, &data_id);
        let response = mercado_pago_webhook_request(
            app.clone(),
            body.as_bytes(),
            &request_id,
            &signature,
        )
        .await
        .unwrap();
        assert_eq!(response, StatusCode::OK, "notification {index}");
    }
    assert_eq!(authorization.lock().unwrap().len(), 1);

    let secretless = TenantCredentialsPayload::MercadoPago(MercadoPagoCredentials {
        access_token: TENANT_TOKEN.into(),
        public_key: Some("tenant-mp-public-key".into()),
        collector_id: Some(4567),
        webhook_secret: None,
    });
    let (key_version, encrypted) = payment_keys.encrypt_payload(1, &secretless).unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE tenant_payment_settings SET credentials=$1,key_version=$2 WHERE tenant_id=1",
        [encrypted.into(), key_version.into()],
    ))
    .await
    .unwrap();
    let rejected = mercado_pago_webhook_request(
        app,
        body.as_bytes(),
        REQUEST_ID,
        &valid_signature,
    )
    .await
    .unwrap();
    assert_eq!(rejected, StatusCode::UNAUTHORIZED);
    assert_eq!(authorization.lock().unwrap().len(), 1);
    let unchanged_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(unchanged_status, "failed");

    server.abort();
    cleanup(root, db, schema).await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL, payment test keyring, and PAYMENT_PROVIDER_MODE=mock"]
async fn late_captured_webhook_reconsumes_available_stock_or_logs_manual_followup() {
    const TENANT_TOKEN: &str = "tc27-tenant-mp-token";
    const TENANT_SECRET: &str = "tc27-tenant-mp-secret";
    const PLATFORM_SECRET: &str = "tc27-platform-mp-secret";
    const TIMESTAMP: &str = "1727280000";

    assert_eq!(
        std::env::var("PAYMENT_PROVIDER_MODE").as_deref().ok(),
        Some("mock")
    );
    let _environment_restore = EnvironmentRestore::capture(&["MP_API_BASE_URL", "MP_WEBHOOK_SECRET"]);
    let payment_keys = Arc::new(PaymentKeyRing::from_env().unwrap());
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let credentials = TenantCredentialsPayload::MercadoPago(MercadoPagoCredentials {
        access_token: TENANT_TOKEN.into(),
        public_key: Some("tc27-tenant-public-key".into()),
        collector_id: Some(4567),
        webhook_secret: Some(TENANT_SECRET.into()),
    });
    let (key_version, encrypted) = payment_keys.encrypt_payload(1, &credentials).unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO tenant_payment_settings (tenant_id,provider,credentials,key_version) VALUES (1,'mercado_pago',$1,$2)",
        [encrypted.into(), key_version.into()],
    ))
    .await
    .unwrap();
    let mut app_state = state(db.clone());
    app_state.payment_keys = payment_keys.clone();
    let app = router(app_state);
    unsafe {
        std::env::set_var("MP_WEBHOOK_SECRET", PLATFORM_SECRET);
    }
    crate::test_log_capture::install();

    for (index, stock_available) in [true, false].into_iter().enumerate() {
        cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
        let purchase_id = create_purchase_from_quote(
            app.clone(),
            &format!("tc27-late-capture-{index}"),
            &[1],
        )
        .await
        .unwrap();
        let (submit_status, submitted) = request(
            app.clone(),
            Method::POST,
            &format!("/purchases/{purchase_id}/payments/submit"),
            Some(json!({
                "token":"mock:pending",
                "paymentMethodId":"visa",
                "installments":1
            })),
            None,
        )
        .await
        .unwrap();
        assert_eq!(submit_status, StatusCode::OK, "{submitted}");
        assert_eq!(submitted["status"], "pending");
        let payment = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT id,amount_cents,attempt_key FROM payment WHERE purchase_id=$1",
                [purchase_id.into()],
            ))
            .await
            .unwrap()
            .unwrap();
        let payment_id: i64 = payment.try_get("", "id").unwrap();
        let amount_cents: i64 = payment.try_get("", "amount_cents").unwrap();
        let attempt_key: String = payment.try_get("", "attempt_key").unwrap();
        let order_id: i64 = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT id FROM orders WHERE purchase_id=$1",
                [purchase_id.into()],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "id")
            .unwrap();
        let provider_id = format!("98765432{index}");
        let external_reference = format!("torg-{purchase_id}-{attempt_key}");
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE payment SET gateway_provider='mercado_pago',gateway_reference=$1 WHERE id=$2",
            [provider_id.clone().into(), payment_id.into()],
        ))
        .await
        .unwrap();
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE purchase SET status='cancelled' WHERE id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap();
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE orders SET status='cancelled',payment_status='failed' WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap();
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE checkout_stock_reservation SET status='released' WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap();
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE sku_stock SET quantity=$1,reserved=0 WHERE tenant_id=1 AND sku_id=1",
            [if stock_available { 10_i32 } else { 0_i32 }.into()],
        ))
        .await
        .unwrap();
        let coupon_code = format!("TC27-LATE-{index}");
        let coupon = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO coupon (uuid,tenant_id,code,coupon_type,value,created_at,updated_at) VALUES (gen_random_uuid(),1,$1,'PERCENTAGE',10,now(),now()) RETURNING id",
                [coupon_code.clone().into()],
            ))
            .await
            .unwrap()
            .unwrap();
        let coupon_id: i64 = coupon.try_get("", "id").unwrap();
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE orders SET coupon_id=$1,coupon_code=$2 WHERE id=$3",
            [coupon_id.into(), coupon_code.into(), order_id.into()],
        ))
        .await
        .unwrap();
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO checkout_coupon_reservation (coupon_id,customer_id,purchase_id,order_id,status,expires_at) VALUES ($1,1,$2,$3,'released',now()-interval '1 hour')",
            [coupon_id.into(), purchase_id.into(), order_id.into()],
        ))
        .await
        .unwrap();

        let authorization = Arc::new(std::sync::Mutex::new(Vec::new()));
        let double = Router::new()
            .route("/v1/payments/{id}", get(mercado_pago_status_double))
            .with_state(MercadoPagoDoubleState {
                authorization: authorization.clone(),
                amount_cents,
                external_reference,
                collector_id: 4567,
                status: "approved".into(),
            });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, double).await.unwrap() });
        unsafe {
            std::env::set_var("MP_API_BASE_URL", format!("http://{address}"));
        }

        let body = format!(r#"{{"data":{{"id":{provider_id}}}}}"#);
        let request_id = format!("tc27-request-{index}");
        let signature = mercado_pago_signature(
            TENANT_SECRET,
            TIMESTAMP,
            &request_id,
            &provider_id,
        );
        crate::test_log_capture::install();
        crate::test_log_capture::clear();
        let response = mercado_pago_webhook_request(
            app.clone(),
            body.as_bytes(),
            &request_id,
            &signature,
        )
        .await
        .unwrap();
        if response != StatusCode::OK {
            panic!(
                "late capture webhook returned {response}; authorization={:?}; logs={}",
                authorization.lock().unwrap(),
                crate::test_log_capture::contents()
            );
        }
        assert_eq!(
            authorization.lock().unwrap().as_slice(),
            &[format!("Bearer {TENANT_TOKEN}")]
        );

        let payment_status: String = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT status FROM payment WHERE id=$1",
                [payment_id.into()],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "status")
            .unwrap();
        let purchase_status: String = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT status FROM purchase WHERE id=$1",
                [purchase_id.into()],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "status")
            .unwrap();
        let order_status: String = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT status FROM orders WHERE purchase_id=$1",
                [purchase_id.into()],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "status")
            .unwrap();
        assert_eq!(payment_status, "captured");
        assert_eq!(purchase_status, "paid");
        assert_eq!(order_status, "paid");
        let reservation_status: String = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT status FROM checkout_stock_reservation WHERE purchase_id=$1",
                [purchase_id.into()],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "status")
            .unwrap();
        let coupon_status: String = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT status FROM checkout_coupon_reservation WHERE purchase_id=$1",
                [purchase_id.into()],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "status")
            .unwrap();
        assert_eq!(coupon_status, "redeemed");
        let stock = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT quantity,reserved FROM sku_stock WHERE tenant_id=1 AND sku_id=1",
                [],
            ))
            .await
            .unwrap()
            .unwrap();
        let quantity: i32 = stock.try_get("", "quantity").unwrap();
        let reserved: i32 = stock.try_get("", "reserved").unwrap();
        assert_eq!(reserved, 0);

        if stock_available {
            assert_eq!(reservation_status, "consumed");
            assert_eq!(quantity, 9);
            let replay = mercado_pago_webhook_request(
                app.clone(),
                body.as_bytes(),
                &request_id,
                &signature,
            )
            .await
            .unwrap();
            assert_eq!(replay, StatusCode::OK);
            let replay_quantity: i32 = db
                .query_one_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "SELECT quantity FROM sku_stock WHERE tenant_id=1 AND sku_id=1",
                    [],
                ))
                .await
                .unwrap()
                .unwrap()
                .try_get("", "quantity")
                .unwrap();
            assert_eq!(replay_quantity, 9);
        } else {
            assert_eq!(reservation_status, "released");
            assert_eq!(quantity, 0);
            assert!(crate::test_log_capture::contents().contains(&format!(
                "ERROR late_capture_unfulfillable: stock unavailable, no refund attempted, manual follow-up, purchase_id={purchase_id}"
            )));
        }
        server.abort();
        let refund_count: i64 = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT count(*) AS count FROM payment_transaction WHERE payment_id=$1 AND operation='refund'",
                [payment_id.into()],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "count")
            .unwrap();
        assert_eq!(refund_count, 0);
    }

    cleanup(root, db, schema).await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL, payment test keyring, and mock setup for initial submit"]
async fn expired_payment_cancel_uses_tenant_mercado_pago_credentials() {
    const TENANT_TOKEN: &str = "tc27-cancel-tenant-token";
    const TENANT_SECRET: &str = "tc27-cancel-tenant-secret";

    let _environment_restore = EnvironmentRestore::capture(&[
        "PAYMENT_PROVIDER_MODE",
        "MP_API_BASE_URL",
        "PAYMENT_RECONCILIATION_WINDOW_HOURS",
    ]);
    unsafe {
        std::env::set_var("PAYMENT_PROVIDER_MODE", "mock");
        std::env::set_var("PAYMENT_RECONCILIATION_WINDOW_HOURS", "3");
    }
    let payment_keys = Arc::new(PaymentKeyRing::from_env().unwrap());
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let credentials = TenantCredentialsPayload::MercadoPago(MercadoPagoCredentials {
        access_token: TENANT_TOKEN.into(),
        public_key: Some("tc27-cancel-public-key".into()),
        collector_id: Some(4567),
        webhook_secret: Some(TENANT_SECRET.into()),
    });
    let (key_version, encrypted) = payment_keys.encrypt_payload(1, &credentials).unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO tenant_payment_settings (tenant_id,provider,credentials,key_version) VALUES (1,'mercado_pago',$1,$2)",
        [encrypted.into(), key_version.into()],
    ))
    .await
    .unwrap();
    let mut app_state = state(db.clone());
    app_state.payment_keys = payment_keys.clone();
    let app = router(app_state);

    cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let purchase_id = create_purchase_from_quote(app.clone(), "tc27-tenant-cancel", &[1])
        .await
        .unwrap();
    let (submit_status, submitted) = request(
        app.clone(),
        Method::POST,
        &format!("/purchases/{purchase_id}/payments/submit"),
        Some(json!({
            "token":"mock:pending",
            "paymentMethodId":"visa",
            "installments":1
        })),
        None,
    )
    .await
    .unwrap();
    assert_eq!(submit_status, StatusCode::OK, "{submitted}");
    let payment = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id,amount_cents,attempt_key FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let payment_id: i64 = payment.try_get("", "id").unwrap();
    let amount_cents: i64 = payment.try_get("", "amount_cents").unwrap();
    let attempt_key: String = payment.try_get("", "attempt_key").unwrap();
    let order_id: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id FROM orders WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "id")
        .unwrap();
    let coupon = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO coupon (uuid,tenant_id,code,coupon_type,value,created_at,updated_at) VALUES (gen_random_uuid(),1,'TC27-CANCEL-CREDS','PERCENTAGE',10,now(),now()) RETURNING id",
            [],
        ))
        .await
        .unwrap()
        .unwrap();
    let coupon_id: i64 = coupon.try_get("", "id").unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE orders SET coupon_id=$1,coupon_code='TC27-CANCEL-CREDS' WHERE id=$2",
        [coupon_id.into(), order_id.into()],
    ))
    .await
    .unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO checkout_coupon_reservation (coupon_id,customer_id,purchase_id,order_id,status,expires_at) VALUES ($1,1,$2,$3,'reserved',now()+interval '1 hour')",
        [coupon_id.into(), purchase_id.into(), order_id.into()],
    ))
    .await
    .unwrap();
    let provider_id = "987650001";
    let external_reference = format!("torg-{purchase_id}-{attempt_key}");
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET gateway_provider='mercado_pago',gateway_reference=$1,created_at=now()-interval '4 hours',updated_at=now()-interval '11 minutes' WHERE id=$2",
        [provider_id.into(), payment_id.into()],
    ))
    .await
    .unwrap();

    let authorization = Arc::new(std::sync::Mutex::new(Vec::new()));
    let double = Router::new()
        .route(
            "/v1/payments/{id}",
            get(mercado_pago_status_double).put(mercado_pago_cancel_double),
        )
        .with_state(MercadoPagoDoubleState {
            authorization: authorization.clone(),
            amount_cents,
            external_reference,
            collector_id: 4567,
            status: "pending".into(),
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, double).await.unwrap() });
    unsafe {
        std::env::set_var("MP_API_BASE_URL", format!("http://{address}"));
        std::env::set_var("PAYMENT_PROVIDER_MODE", "real");
    }

    let mut reconciliation_state = state(db.clone());
    reconciliation_state.payment_keys = payment_keys;
    let handled = crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(
        &reconciliation_state,
    )
    .await
    .unwrap();
    assert_eq!(handled, 1);
    assert_eq!(
        authorization.lock().unwrap().as_slice(),
        &[
            format!("Bearer {TENANT_TOKEN}"),
            format!("Bearer {TENANT_TOKEN}")
        ]
    );
    let payment_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM payment WHERE id=$1",
            [payment_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(payment_status, "failed");
    let purchase_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM purchase WHERE id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(purchase_status, "payment_failed");
    let order_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM orders WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(order_status, "payment_failed");
    let stock_reservation_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM checkout_stock_reservation WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(stock_reservation_status, "released");
    let coupon_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM checkout_coupon_reservation WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(coupon_status, "released");
    server.abort();
    cleanup(root, db, schema).await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL, payment test keyring, and local provider double"]
async fn mercado_pago_provider_error_logs_redact_bodies_across_payment_flows() {
    const TENANT_TOKEN: &str = "tc28-tenant-mp-token";
    const TENANT_SECRET: &str = "tc28-tenant-mp-secret";
    const PROVIDER_ID: &str = "987650028";
    const REQUEST_ID: &str = "tc28-request-id";
    const TIMESTAMP: &str = "1727280000";
    let markers = [
        "tc28-token-body-marker",
        "tc28-tax-body-marker",
        "tc28-card-body-marker",
    ];

    let _environment_restore = EnvironmentRestore::capture(&[
        "PAYMENT_PROVIDER_MODE",
        "MP_API_BASE_URL",
        "PAYMENT_RECONCILIATION_WINDOW_HOURS",
    ]);
    unsafe {
        std::env::set_var("PAYMENT_PROVIDER_MODE", "real");
        std::env::set_var("PAYMENT_RECONCILIATION_WINDOW_HOURS", "3");
    }
    let payment_keys = Arc::new(PaymentKeyRing::from_env().unwrap());
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let credentials = TenantCredentialsPayload::MercadoPago(MercadoPagoCredentials {
        access_token: TENANT_TOKEN.into(),
        public_key: Some("tc28-tenant-public-key".into()),
        collector_id: Some(4567),
        webhook_secret: Some(TENANT_SECRET.into()),
    });
    let (key_version, encrypted) = payment_keys.encrypt_payload(1, &credentials).unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO tenant_payment_settings (tenant_id,provider,credentials,key_version) VALUES (1,'mercado_pago',$1,$2)",
        [encrypted.into(), key_version.into()],
    ))
    .await
    .unwrap();
    let mut app_state = state(db.clone());
    app_state.payment_keys = payment_keys.clone();
    let app = router(app_state);

    let double = Router::new()
        .route("/v1/payments", post(tc28_provider_error_response))
        .route("/v1/payments/search", get(tc28_provider_error_response))
        .route("/v1/payments/{id}", get(tc28_provider_error_response));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, double).await.unwrap() });
    unsafe {
        std::env::set_var("MP_API_BASE_URL", format!("http://{address}"));
    }
    crate::test_log_capture::install();

    cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let purchase_id = create_purchase_from_quote(app.clone(), "tc28-redaction", &[1])
        .await
        .unwrap();
    crate::test_log_capture::clear();
    let (submit_status, submitted) = request(
        app.clone(),
        Method::POST,
        &format!("/purchases/{purchase_id}/payments/submit"),
        Some(json!({
            "token":"tc28-input-card-token-marker",
            "paymentMethodId":"visa",
            "installments":1
        })),
        None,
    )
    .await
    .unwrap();
    assert_eq!(submit_status, StatusCode::OK, "{submitted}");
    let submit_logs = crate::test_log_capture::contents();
    for marker in markers.iter().chain(["tc28-input-card-token-marker"].iter()) {
        assert!(!submit_logs.contains(marker), "{submit_logs}");
    }

    let payment_id: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "id")
        .unwrap();
    crate::test_log_capture::clear();
    let (status_code, _) = request(
        app.clone(),
        Method::GET,
        &format!("/purchases/{purchase_id}/payments/status"),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(status_code, StatusCode::SERVICE_UNAVAILABLE);
    let status_logs = crate::test_log_capture::contents();
    for marker in markers {
        assert!(!status_logs.contains(marker), "{status_logs}");
    }

    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET gateway_provider='mercado_pago',gateway_reference=$1 WHERE id=$2",
        [PROVIDER_ID.into(), payment_id.into()],
    ))
    .await
    .unwrap();
    let body = format!(r#"{{"data":{{"id":"{PROVIDER_ID}"}}}}"#);
    let signature = mercado_pago_signature(
        TENANT_SECRET,
        TIMESTAMP,
        REQUEST_ID,
        PROVIDER_ID,
    );
    crate::test_log_capture::clear();
    let webhook_status = mercado_pago_webhook_request(
        app.clone(),
        body.as_bytes(),
        REQUEST_ID,
        &signature,
    )
    .await
    .unwrap();
    assert_eq!(webhook_status, StatusCode::SERVICE_UNAVAILABLE);
    let webhook_logs = crate::test_log_capture::contents();
    for marker in markers {
        assert!(!webhook_logs.contains(marker), "{webhook_logs}");
    }

    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET created_at=now()-interval '1 hour',updated_at=now()-interval '11 minutes' WHERE id=$1",
        [payment_id.into()],
    ))
    .await
    .unwrap();
    crate::test_log_capture::clear();
    let mut reconciliation_state = state(db.clone());
    reconciliation_state.payment_keys = payment_keys;
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&reconciliation_state)
            .await
            .unwrap(),
        0
    );
    let reconciliation_logs = crate::test_log_capture::contents();
    assert!(reconciliation_logs.contains("WARN reconciliation: payment"));
    for marker in markers {
        assert!(!reconciliation_logs.contains(marker), "{reconciliation_logs}");
    }
    assert_payment_markers_absent(
        &db,
        &[
            "tc28-token-body-marker",
            "tc28-tax-body-marker",
            "tc28-card-body-marker",
            "tc28-input-card-token-marker",
        ],
    )
    .await;

    server.abort();
    cleanup(root, db, schema).await.unwrap();
}

async fn tc28_pagbank_error_response() -> (StatusCode, axum::Json<Value>) {
    (
        StatusCode::BAD_GATEWAY,
        axum::Json(json!({
            "error_messages": [{
                "code": "50099",
                "description": "token=tc28-pagbank-token-marker tax_id=tc28-pagbank-tax-marker card=tc28-pagbank-card-marker"
            }]
        })),
    )
}

async fn tc01_non_card_charge_response(
    axum::Json(request): axum::Json<Value>,
) -> axum::Json<Value> {
    axum::Json(json!({
        "id": "123456789",
        "status": "approved",
        "transaction_amount": request.get("transaction_amount").cloned().unwrap_or(Value::Null),
        "currency_id": "BRL",
        "external_reference": request.get("external_reference").cloned().unwrap_or(Value::Null),
        "collector_id": 4567,
        "payment_type_id": "debit_card",
        "payment_method_id": "visa"
    }))
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL, payment test keyring, and local PagBank double"]
async fn pagbank_provider_error_logs_redact_bodies_across_payment_flows() {
    const TENANT_TOKEN: &str = "tc28-pagbank-tenant-token";
    const MARKERS: [&str; 3] = [
        "tc28-pagbank-token-marker",
        "tc28-pagbank-tax-marker",
        "tc28-pagbank-card-marker",
    ];
    let _environment_restore = EnvironmentRestore::capture(&[
        "PAYMENT_PROVIDER_MODE",
        "PAGSEGURO_API_BASE_URL",
        "PAYMENT_RECONCILIATION_WINDOW_HOURS",
    ]);
    unsafe {
        std::env::set_var("PAYMENT_PROVIDER_MODE", "real");
        std::env::set_var("PAYMENT_RECONCILIATION_WINDOW_HOURS", "3");
    }
    let payment_keys = Arc::new(PaymentKeyRing::from_env().unwrap());
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let credentials = TenantCredentialsPayload::PagSeguro(PagSeguroCredentials {
        token: TENANT_TOKEN.into(),
        public_key: None,
        environment: Some("sandbox".into()),
    });
    let (key_version, encrypted) = payment_keys.encrypt_payload(1, &credentials).unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO tenant_payment_settings (tenant_id,provider,credentials,key_version) VALUES (1,'pagseguro',$1,$2)",
        [encrypted.into(), key_version.into()],
    ))
    .await
    .unwrap();
    let mut app_state = state(db.clone());
    app_state.payment_keys = payment_keys.clone();
    let app = router(app_state);

    let double = Router::new()
        .route("/charges", post(tc28_pagbank_error_response).get(tc28_pagbank_error_response))
        .route("/charges/{reference}", get(tc28_pagbank_error_response));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let base_url = format!("http://{address}");
    let server = tokio::spawn(async move { axum::serve(listener, double).await.unwrap() });
    unsafe {
        std::env::set_var("PAGSEGURO_API_BASE_URL", &base_url);
    }
    crate::test_log_capture::install();

    cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let purchase_id = create_purchase_from_quote(app.clone(), "tc28-pagbank-redaction", &[1])
        .await
        .unwrap();
    crate::test_log_capture::clear();
    let (submit_status, submitted) = request(
        app.clone(),
        Method::POST,
        &format!("/purchases/{purchase_id}/payments/submit"),
        Some(json!({
            "token":"tc28-input-pagbank-card-token-marker",
            "paymentMethodId":"visa",
            "installments":1
        })),
        None,
    )
    .await
    .unwrap();
    assert_eq!(submit_status, StatusCode::OK, "{submitted}");
    let submit_logs = crate::test_log_capture::contents();
    for marker in MARKERS.iter().chain(["tc28-input-pagbank-card-token-marker"].iter()) {
        assert!(!submit_logs.contains(marker), "{submit_logs}");
    }

    let payment = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id,attempt_key FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let payment_id: i64 = payment.try_get("", "id").unwrap();
    let attempt_key: String = payment.try_get("", "attempt_key").unwrap();
    crate::test_log_capture::clear();
    let (status_code, _) = request(
        app.clone(),
        Method::GET,
        &format!("/purchases/{purchase_id}/payments/status"),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(status_code, StatusCode::SERVICE_UNAVAILABLE);
    let status_logs = crate::test_log_capture::contents();
    for marker in MARKERS {
        assert!(!status_logs.contains(marker), "{status_logs}");
    }

    let body = format!(
        r#"{{"reference_id":"torg-{purchase_id}-{attempt_key}","status":"PAID"}}"#
    );
    let signature = pagbank_signature(TENANT_TOKEN, body.as_bytes());
    crate::test_log_capture::clear();
    let webhook_status = webhook_request_with_provider_base(
        app.clone(),
        body.as_bytes(),
        &signature,
        Some(&base_url),
    )
    .await
    .unwrap();
    assert_eq!(webhook_status, StatusCode::SERVICE_UNAVAILABLE);
    let webhook_logs = crate::test_log_capture::contents();
    for marker in MARKERS {
        assert!(!webhook_logs.contains(marker), "{webhook_logs}");
    }

    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET created_at=now()-interval '1 hour',updated_at=now()-interval '11 minutes' WHERE id=$1",
        [payment_id.into()],
    ))
    .await
    .unwrap();
    crate::test_log_capture::clear();
    let mut reconciliation_state = state(db.clone());
    reconciliation_state.payment_keys = payment_keys;
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&reconciliation_state)
            .await
            .unwrap(),
        0
    );
    let reconciliation_logs = crate::test_log_capture::contents();
    assert!(reconciliation_logs.contains("WARN reconciliation: payment"));
    for marker in MARKERS {
        assert!(!reconciliation_logs.contains(marker), "{reconciliation_logs}");
    }
    assert_payment_markers_absent(
        &db,
        &[
            "tc28-pagbank-token-marker",
            "tc28-pagbank-tax-marker",
            "tc28-pagbank-card-marker",
            "tc28-input-pagbank-card-token-marker",
        ],
    )
    .await;

    server.abort();
    cleanup(root, db, schema).await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL and payment test keyring"]
async fn tc01_card_submit_rejects_invalid_input_and_non_card_provider() {
    let _environment_restore =
        EnvironmentRestore::capture(&["PAYMENT_PROVIDER_MODE", "MP_API_BASE_URL"]);
    unsafe {
        std::env::set_var("PAYMENT_PROVIDER_MODE", "real");
    }
    let payment_keys = Arc::new(PaymentKeyRing::from_env().unwrap());
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let credentials = TenantCredentialsPayload::MercadoPago(MercadoPagoCredentials {
        access_token: "tc01-tenant-mp-token".into(),
        public_key: Some("tc01-tenant-public-key".into()),
        collector_id: Some(4567),
        webhook_secret: Some("tc01-tenant-mp-secret".into()),
    });
    let (key_version, encrypted) = payment_keys.encrypt_payload(1, &credentials).unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO tenant_payment_settings (tenant_id,provider,credentials,key_version) VALUES (1,'mercado_pago',$1,$2)",
        [encrypted.into(), key_version.into()],
    ))
    .await
    .unwrap();
    let mut app_state = state(db.clone());
    app_state.payment_keys = payment_keys;
    let app = router(app_state);
    cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let purchase_id = create_purchase_from_quote(app.clone(), "tc01-card-limits", &[1])
        .await
        .unwrap();
    let submit_uri = format!("/purchases/{purchase_id}/payments/submit");

    for invalid in [
        json!({
            "token":"card-token",
            "paymentMethodId":"visa",
            "installments":1,
            "cardNumber":"4111111111111111"
        }),
        json!({"token":"card-token","paymentMethodId":"visa","installments":0}),
        json!({"token":"card-token","paymentMethodId":"visa","installments":25}),
        json!({"token":"x".repeat(1025),"paymentMethodId":"visa","installments":1}),
    ] {
        let status = request_in(
            app.clone(),
            Method::POST,
            &submit_uri,
            Some(invalid),
            TenantContext::Marketplace,
        )
        .await
        .unwrap();
        assert!(status.is_client_error(), "{status}");
    }
    let attempts: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT count(*) AS count FROM payment_transaction WHERE payment_id=(SELECT id FROM payment WHERE purchase_id=$1)",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "count")
        .unwrap();
    assert_eq!(attempts, 0);

    let double = Router::new()
        .route("/v1/payments", post(tc01_non_card_charge_response));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, double).await.unwrap() });
    unsafe {
        std::env::set_var("MP_API_BASE_URL", format!("http://{address}"));
    }
    let status = request_in(
        app,
        Method::POST,
        &submit_uri,
        Some(json!({"token":"valid-token","paymentMethodId":"visa","installments":1})),
        TenantContext::Marketplace,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    let payment_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(payment_status, "pending");
    let attempts: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT count(*) AS count FROM payment_transaction WHERE payment_id=(SELECT id FROM payment WHERE purchase_id=$1)",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "count")
        .unwrap();
    assert_eq!(attempts, 1);

    server.abort();
    cleanup(root, db, schema).await.unwrap();
}

#[derive(Clone, Default)]
struct Tc04ChargeDoubleState {
    requests: Arc<std::sync::Mutex<Vec<Value>>>,
}

async fn tc04_charge_double(
    axum::extract::State(state): axum::extract::State<Tc04ChargeDoubleState>,
    axum::Json(request): axum::Json<Value>,
) -> axum::Json<Value> {
    state.requests.lock().unwrap().push(request.clone());
    axum::Json(json!({
        "id": "987654321",
        "status": "approved",
        "transaction_amount": request["transaction_amount"],
        "currency_id": "BRL",
        "external_reference": request["external_reference"],
        "collector_id": 4567,
        "payment_type_id": "credit_card",
        "payment_method_id": "visa",
        "card": {
            "last_four_digits": "4242",
            "expiration_month": 12,
            "expiration_year": 2030,
            "cardholder": {"name": "Mock Test Card"}
        }
    }))
}

#[derive(Clone, Default)]
struct Tc05BlockingChargeState {
    requests: Arc<std::sync::Mutex<Vec<Value>>>,
    idempotency_keys: Arc<std::sync::Mutex<Vec<String>>>,
    entered: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Notify>,
}

async fn tc05_mercado_pago_charge_double(
    axum::extract::State(state): axum::extract::State<Tc05BlockingChargeState>,
    headers: HeaderMap,
    axum::Json(request): axum::Json<Value>,
) -> axum::Json<Value> {
    state.requests.lock().unwrap().push(request.clone());
    state.idempotency_keys.lock().unwrap().push(
        headers
            .get("x-idempotency-key")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned(),
    );
    state.entered.notify_one();
    state.release.notified().await;
    axum::Json(json!({
        "id": "987650500",
        "status": "approved",
        "transaction_amount": request["transaction_amount"],
        "currency_id": "BRL",
        "external_reference": request["external_reference"],
        "collector_id": 4567,
        "payment_type_id": "credit_card",
        "payment_method_id": "visa",
        "card": {"last_four_digits":"4242"}
    }))
}

async fn tc05_pagbank_charge_double(
    axum::extract::State(state): axum::extract::State<Tc05BlockingChargeState>,
    headers: HeaderMap,
    axum::Json(request): axum::Json<Value>,
) -> axum::Json<Value> {
    state.requests.lock().unwrap().push(request.clone());
    state.idempotency_keys.lock().unwrap().push(
        headers
            .get("x-idempotency-key")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned(),
    );
    state.entered.notify_one();
    state.release.notified().await;
    axum::Json(json!({
        "id": "CHAR_TC05_001",
        "reference_id": request["reference_id"],
        "status": "PAID",
        "amount": request["amount"],
        "payment_method": {
            "type": "CREDIT_CARD",
            "card": {"brand":"visa","last_digits":"4242","exp_month":12,"exp_year":2030}
        }
    }))
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL and payment test keyring"]
async fn tc04_charge_uses_purchase_amount_and_payer_and_skips_zero_total() {
    let _environment_restore =
        EnvironmentRestore::capture(&["PAYMENT_PROVIDER_MODE", "MP_API_BASE_URL"]);
    unsafe {
        std::env::set_var("PAYMENT_PROVIDER_MODE", "real");
    }
    let payment_keys = Arc::new(PaymentKeyRing::from_env().unwrap());
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let credentials = TenantCredentialsPayload::MercadoPago(MercadoPagoCredentials {
        access_token: "tc04-tenant-mp-token".into(),
        public_key: Some("tc04-tenant-public-key".into()),
        collector_id: Some(4567),
        webhook_secret: Some("tc04-tenant-mp-secret".into()),
    });
    let (key_version, encrypted) = payment_keys.encrypt_payload(1, &credentials).unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO tenant_payment_settings (tenant_id,provider,credentials,key_version) VALUES (1,'mercado_pago',$1,$2)",
        [encrypted.into(), key_version.into()],
    ))
    .await
    .unwrap();
    let mut app_state = state(db.clone());
    app_state.payment_keys = payment_keys;
    let app = router(app_state);
    let requests = Arc::new(std::sync::Mutex::new(Vec::new()));
    let double = Router::new()
        .route("/v1/payments", post(tc04_charge_double))
        .with_state(Tc04ChargeDoubleState {
            requests: requests.clone(),
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, double).await.unwrap() });
    unsafe {
        std::env::set_var("MP_API_BASE_URL", format!("http://{address}"));
    }

    cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let purchase_id = create_purchase_from_quote(app.clone(), "tc04-known-total", &[1])
        .await
        .unwrap();
    let submit_uri = format!("/purchases/{purchase_id}/payments/submit");
    let status = request_in(
        app.clone(),
        Method::POST,
        &submit_uri,
        Some(json!({
            "token":"tc04-card-token",
            "paymentMethodId":"visa",
            "installments":3,
            "amount":1,
            "payerEmail":"attacker@example.test",
            "payerTaxId":"00000000000"
        })),
        TenantContext::Marketplace,
    )
    .await
    .unwrap();
    assert!(status.is_client_error(), "{status}");
    assert!(requests.lock().unwrap().is_empty());

    let expected = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT p.total_cents,p.customer_email,p.customer_tax_id FROM purchase p WHERE p.id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let expected_amount_cents: i64 = expected.try_get("", "total_cents").unwrap();
    let expected_email: String = expected.try_get("", "customer_email").unwrap();
    let expected_tax_id: String = expected.try_get("", "customer_tax_id").unwrap();
    let payment_amount_cents: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT amount_cents FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "amount_cents")
        .unwrap();
    let (status, submitted) = request(
        app.clone(),
        Method::POST,
        &submit_uri,
        Some(json!({
            "token":"tc04-card-token",
            "paymentMethodId":"visa",
            "installments":3
        })),
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{submitted}");
    let captured_requests = requests.lock().unwrap();
    assert_eq!(captured_requests.len(), 1);
    let provider_request = &captured_requests[0];
    assert_eq!(
        provider_request["transaction_amount"],
        payment_amount_cents as f64 / 100.0
    );
    assert_eq!(payment_amount_cents, expected_amount_cents);
    assert_eq!(provider_request["payer"]["email"], expected_email);
    assert_eq!(
        provider_request["payer"]["identification"]["number"],
        expected_tax_id
    );
    assert_eq!(provider_request["installments"], 3);
    drop(captured_requests);

    db.execute_unprepared(
        "UPDATE sku SET price_cents=0 WHERE id=1; UPDATE shipping_rate SET price_cents=0 WHERE tenant_id=1",
    )
    .await
    .unwrap();
    cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let zero_purchase_id = create_purchase_from_quote(app.clone(), "tc04-zero-total", &[1])
        .await
        .unwrap();
    let (zero_status, zero_payment) = request(
        app,
        Method::POST,
        &format!("/purchases/{zero_purchase_id}/payments/submit"),
        Some(json!({
            "token":"tc04-zero-token",
            "paymentMethodId":"visa",
            "installments":1
        })),
        None,
    )
    .await
    .unwrap();
    assert_eq!(zero_status, StatusCode::OK, "{zero_payment}");
    assert_eq!(requests.lock().unwrap().len(), 1);
    let zero_transactions: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT count(*) AS count FROM payment_transaction WHERE payment_id=(SELECT id FROM payment WHERE purchase_id=$1)",
            [zero_purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "count")
        .unwrap();
    assert_eq!(zero_transactions, 0);

    server.abort();
    cleanup(root, db, schema).await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL, payment test keyring and local provider doubles"]
async fn tc05_persists_attempt_before_provider_and_reuses_it_for_both_providers() {
    let _environment_restore = EnvironmentRestore::capture(&[
        "PAYMENT_PROVIDER_MODE",
        "MP_API_BASE_URL",
        "PAGSEGURO_API_BASE_URL",
    ]);
    unsafe {
        std::env::set_var("PAYMENT_PROVIDER_MODE", "real");
    }
    let payment_keys = Arc::new(PaymentKeyRing::from_env().unwrap());
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let mut app_state = state(db.clone());
    app_state.payment_keys = payment_keys.clone();
    let app = router(app_state);

    for provider_name in ["mercado_pago", "pagseguro"] {
        let credentials = if provider_name == "mercado_pago" {
            TenantCredentialsPayload::MercadoPago(MercadoPagoCredentials {
                access_token: "tc05-tenant-mp-token".into(),
                public_key: Some("tc05-tenant-mp-public".into()),
                collector_id: Some(4567),
                webhook_secret: Some("tc05-tenant-mp-secret".into()),
            })
        } else {
            TenantCredentialsPayload::PagSeguro(PagSeguroCredentials {
                token: "tc05-tenant-pagbank-token".into(),
                public_key: None,
                environment: Some("sandbox".into()),
            })
        };
        let (key_version, encrypted) = payment_keys.encrypt_payload(1, &credentials).unwrap();
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO tenant_payment_settings (tenant_id,provider,credentials,key_version) VALUES (1,$1,$2,$3) ON CONFLICT (tenant_id) DO UPDATE SET provider=EXCLUDED.provider,credentials=EXCLUDED.credentials,key_version=EXCLUDED.key_version",
            [provider_name.into(), encrypted.into(), key_version.into()],
        ))
        .await
        .unwrap();

        cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
        let purchase_id = create_purchase_from_quote(
            app.clone(),
            &format!("tc05-{provider_name}"),
            &[1],
        )
        .await
        .unwrap();
        let submit_uri = format!("/purchases/{purchase_id}/payments/submit");
        let charge_state = Tc05BlockingChargeState::default();
        let double = if provider_name == "mercado_pago" {
            Router::new()
                .route("/v1/payments", post(tc05_mercado_pago_charge_double))
                .with_state(charge_state.clone())
        } else {
            Router::new()
                .route("/charges", post(tc05_pagbank_charge_double))
                .with_state(charge_state.clone())
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, double).await.unwrap() });
        unsafe {
            if provider_name == "mercado_pago" {
                std::env::set_var("MP_API_BASE_URL", format!("http://{address}"));
            } else {
                std::env::set_var("PAGSEGURO_API_BASE_URL", format!("http://{address}"));
            }
        }

        let first_app = app.clone();
        let first_uri = submit_uri.clone();
        let first_submission = tokio::spawn(async move {
            request(
                first_app,
                Method::POST,
                &first_uri,
                Some(json!({
                    "token":"tc05-provider-token",
                    "paymentMethodId":"visa",
                    "installments":2
                })),
                None,
            )
            .await
        });
        charge_state.entered.notified().await;

        let attempt = db
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT p.status,p.attempt_key,t.status AS transaction_status,t.idempotency_key FROM payment p JOIN payment_transaction t ON t.payment_id=p.id WHERE p.purchase_id=$1",
                [purchase_id.into()],
            ))
            .await
            .unwrap()
            .unwrap();
        let payment_status: String = attempt.try_get("", "status").unwrap();
        let attempt_key: String = attempt.try_get("", "attempt_key").unwrap();
        let transaction_status: String = attempt.try_get("", "transaction_status").unwrap();
        let transaction_key: String = attempt.try_get("", "idempotency_key").unwrap();
        assert_eq!(payment_status, "pending");
        assert_eq!(transaction_status, "pending");
        assert!(!attempt_key.is_empty());
        assert_eq!(transaction_key, attempt_key);
        assert_eq!(charge_state.requests.lock().unwrap().len(), 1);
        assert_eq!(
            charge_state.idempotency_keys.lock().unwrap().as_slice(),
            &[attempt_key.clone()]
        );

        let (pending_repeat_status, pending_repeat) = request(
            app.clone(),
            Method::POST,
            &submit_uri,
            Some(json!({
                "token":"different-token",
                "paymentMethodId":"visa",
                "installments":1
            })),
            None,
        )
        .await
        .unwrap();
        assert_eq!(pending_repeat_status, StatusCode::OK, "{pending_repeat}");
        assert_eq!(pending_repeat["status"], "pending");
        assert_eq!(charge_state.requests.lock().unwrap().len(), 1);

        charge_state.release.notify_one();
        let (first_status, first_response) = first_submission.await.unwrap().unwrap();
        assert_eq!(first_status, StatusCode::OK, "{first_response}");
        assert_eq!(first_response["status"], "captured");
        let (captured_repeat_status, captured_repeat) = request(
            app.clone(),
            Method::POST,
            &submit_uri,
            Some(json!({
                "token":"different-token",
                "paymentMethodId":"visa",
                "installments":1
            })),
            None,
        )
        .await
        .unwrap();
        assert_eq!(captured_repeat_status, StatusCode::OK, "{captured_repeat}");
        assert_eq!(captured_repeat["status"], "captured");
        assert_eq!(charge_state.requests.lock().unwrap().len(), 1);
        server.abort();
    }

    cleanup(root, db, schema).await.unwrap();
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL and PAYMENT_PROVIDER_MODE=mock"]
async fn tc03_persists_only_accepted_card_metadata() {
    let _environment_restore = EnvironmentRestore::capture(&[
        "PAYMENT_PROVIDER_MODE",
        "PAYMENT_MOCK_PROVIDER",
    ]);
    unsafe {
        std::env::set_var("PAYMENT_PROVIDER_MODE", "mock");
        std::env::set_var("PAYMENT_MOCK_PROVIDER", "mercado_pago");
    }
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let app = router(state(db.clone()));
    cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let purchase_id = create_purchase_from_quote(app.clone(), "tc03-card-metadata", &[1])
        .await
        .unwrap();
    let (status, submitted) = request(
        app,
        Method::POST,
        &format!("/purchases/{purchase_id}/payments/submit"),
        Some(json!({
            "token":"mock:approved",
            "paymentMethodId":"visa",
            "installments":1
        })),
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{submitted}");
    assert_eq!(submitted["status"], "captured");

    let metadata = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT cardholder_name,brand,last_four_digits,expiration_month,expiration_year FROM credit_card_details WHERE payment_id=(SELECT id FROM payment WHERE purchase_id=$1)",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let holder: String = metadata.try_get("", "cardholder_name").unwrap();
    let brand: String = metadata.try_get("", "brand").unwrap();
    let last_four: String = metadata.try_get("", "last_four_digits").unwrap();
    let expiration_month: i32 = metadata.try_get("", "expiration_month").unwrap();
    let expiration_year: i32 = metadata.try_get("", "expiration_year").unwrap();
    assert_eq!(holder, "Mock Test Card");
    assert_eq!(brand, "visa");
    assert_eq!(last_four, "4242");
    assert_eq!(expiration_month, 12);
    assert_eq!(expiration_year, 2030);

    let forbidden_columns: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT count(*) AS count FROM information_schema.columns WHERE table_schema=current_schema() AND table_name='credit_card_details' AND column_name IN ('card_number','pan','cvv','cvc','token')",
            [],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "count")
        .unwrap();
    assert_eq!(forbidden_columns, 0);

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
async fn reconciliation_covers_pending_authorized_backoff_locking_and_cancellation() {
    // NOV-5 TC-11/23/24: SR-PAY-021, SR-PAY-023
    let _environment_restore =
        EnvironmentRestore::capture(&["PAYMENT_RECONCILIATION_WINDOW_HOURS"]);
    unsafe {
        std::env::set_var("PAYMENT_RECONCILIATION_WINDOW_HOURS", "3");
    }
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
    let payment_id: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "id")
        .unwrap();
    let coupon_row = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO coupon (uuid,tenant_id,code,coupon_type,value,created_at,updated_at) VALUES (gen_random_uuid(),1,'TC27-RELEASE','PERCENTAGE',10,now(),now()) RETURNING id",
            [],
        ))
        .await
        .unwrap()
        .unwrap();
    let coupon_id: i64 = coupon_row.try_get("", "id").unwrap();
    let order_id: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id FROM orders WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "id")
        .unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO checkout_coupon_reservation (coupon_id,customer_id,purchase_id,order_id,status,expires_at) VALUES ($1,1,$2,$3,'reserved',now()+interval '1 hour')",
        [coupon_id.into(), purchase_id.into(), order_id.into()],
    ))
    .await
    .unwrap();

    cart_fixture(&db, Some(1), 1, &[1]).await.unwrap();
    let captured_purchase_id = create_purchase_from_quote(app.clone(), "recon-captured", &[1])
        .await
        .unwrap();
    let (captured_status, captured_submit) = request(
        app.clone(),
        Method::POST,
        &format!("/purchases/{captured_purchase_id}/payments/submit"),
        Some(json!({"token":"mock:pending","paymentMethodId":"visa","installments":1})),
        None,
    )
    .await
    .unwrap();
    assert_eq!(captured_status, StatusCode::OK, "{captured_submit}");
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET gateway_reference=replace(gateway_reference, ':pending:', ':captured:'), created_at=now()-interval '1 hour', updated_at=now()-interval '11 minutes' WHERE purchase_id=$1",
        [captured_purchase_id.into()],
    ))
    .await
    .unwrap();
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        1
    );
    let reconciled_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM payment WHERE purchase_id=$1",
            [captured_purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(reconciled_status, "captured");

    // Fresh payment and a payment inside the two-minute age band are not due yet.
    let claimed = crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
        .await
        .unwrap();
    assert_eq!(claimed, 0);
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET created_at=now()-interval '10 minutes', updated_at=now()-interval '60 seconds' WHERE purchase_id=$1",
        [purchase_id.into()],
    ))
    .await
    .unwrap();
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        0
    );
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET updated_at=now()-interval '3 minutes' WHERE purchase_id=$1",
        [purchase_id.into()],
    ))
    .await
    .unwrap();
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        0
    );

    // The oldest age band waits ten minutes between polls.
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET created_at=now()-interval '1 hour', updated_at=now()-interval '5 minutes' WHERE purchase_id=$1",
        [purchase_id.into()],
    ))
    .await
    .unwrap();
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        0
    );
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET updated_at=now()-interval '11 minutes' WHERE purchase_id=$1",
        [purchase_id.into()],
    ))
    .await
    .unwrap();
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        1
    );

    // A four-hour-old payment is polled, not cancelled, with a six-hour override.
    unsafe {
        std::env::set_var("PAYMENT_RECONCILIATION_WINDOW_HOURS", "6");
    }
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET created_at=now()-interval '4 hours', updated_at=now()-interval '11 minutes' WHERE purchase_id=$1",
        [purchase_id.into()],
    ))
    .await
    .unwrap();
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        1
    );
    let pending_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(pending_status, "pending");
    unsafe {
        std::env::set_var("PAYMENT_RECONCILIATION_WINDOW_HOURS", "3");
    }

    // A concurrent row lock is skipped rather than polled twice or blocked on.
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET created_at=now()-interval '20 minutes', updated_at=now()-interval '11 minutes' WHERE purchase_id=$1",
        [purchase_id.into()],
    ))
    .await
    .unwrap();
    let lock = db.begin().await.unwrap();
    lock.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET updated_at=updated_at WHERE id=$1",
        [payment_id.into()],
    ))
    .await
    .unwrap();
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        0
    );
    lock.commit().await.unwrap();
    let (first_worker, second_worker) = tokio::join!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state),
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state),
    );
    assert_eq!(first_worker.unwrap() + second_worker.unwrap(), 1);
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        0
    );

    // Provider errors are warned and leave the payment pending.
    crate::test_log_capture::install();
    crate::test_log_capture::clear();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET gateway_reference=replace(gateway_reference, ':pending:', ':error:'), created_at=now()-interval '1 hour', updated_at=now()-interval '11 minutes' WHERE purchase_id=$1",
        [purchase_id.into()],
    ))
    .await
    .unwrap();
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        0
    );
    let captured_logs = crate::test_log_capture::contents();
    assert!(captured_logs.contains(&format!(
        "WARN reconciliation: payment {payment_id} provider error: mock provider error requested"
    )));
    let pending_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(pending_status, "pending");

    crate::test_log_capture::clear();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET status='authorized', created_at=now()-interval '1 hour', updated_at=now()-interval '11 minutes' WHERE purchase_id=$1",
        [purchase_id.into()],
    ))
    .await
    .unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment_transaction SET status='authorized' WHERE payment_id=$1",
        [payment_id.into()],
    ))
    .await
    .unwrap();
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        0
    );
    let authorized_logs = crate::test_log_capture::contents();
    assert!(!authorized_logs.contains(&format!(
        "WARN reconciliation: payment {payment_id} provider error"
    )));
    let authorized_status: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM payment WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(authorized_status, "authorized");

    // Past the 3h window: cancelled at the provider, purchase fails, no refund.
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET status='pending', gateway_reference=replace(gateway_reference, ':error:', ':pending:'), created_at=now()-interval '4 hours', updated_at=now()-interval '11 minutes' WHERE purchase_id=$1",
        [purchase_id.into()],
    ))
    .await
    .unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment_transaction SET status='pending' WHERE payment_id=$1",
        [payment_id.into()],
    ))
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
    let transaction = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT operation,status FROM payment_transaction WHERE payment_id=$1",
            [payment_id.into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let operation: String = transaction.try_get("", "operation").unwrap();
    let transaction_status: String = transaction.try_get("", "status").unwrap();
    assert_eq!(operation, "charge");
    assert_eq!(transaction_status, "failed");
    let refund_count: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT count(*) AS count FROM payment_transaction WHERE payment_id=$1 AND operation='refund'",
            [payment_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "count")
        .unwrap();
    assert_eq!(refund_count, 0);
    let released_stock: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM checkout_stock_reservation WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(released_stock, "released");
    let released_coupon: String = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status FROM checkout_coupon_reservation WHERE purchase_id=$1",
            [purchase_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "status")
        .unwrap();
    assert_eq!(released_coupon, "released");

    // Resolved payments are not claimed again.
    assert_eq!(
        crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
            .await
            .unwrap(),
        0
    );
    let reserved_stock: i32 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT reserved FROM sku_stock WHERE tenant_id=1 AND sku_id=1",
            [],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "reserved")
        .unwrap();
    assert_eq!(reserved_stock, 0);

    // A provider-reported late capture cannot be cancelled or refunded.
    let payment_row = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT attempt_key,amount_cents FROM payment WHERE id=$1",
            [payment_id.into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let attempt_key: String = payment_row.try_get("", "attempt_key").unwrap();
    let payment_amount_cents: i64 = payment_row.try_get("", "amount_cents").unwrap();
    let captured_reference = format!(
        "mock:mercado_pago:captured:{payment_amount_cents}:BRL:torg-{purchase_id}-{attempt_key}"
    );
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment SET status='pending',gateway_reference=$1,created_at=now()-interval '4 hours',updated_at=now()-interval '11 minutes' WHERE id=$2",
        [captured_reference.into(), payment_id.into()],
    ))
    .await
    .unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE payment_transaction SET status='pending' WHERE payment_id=$1",
        [payment_id.into()],
    ))
    .await
    .unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE sku_stock SET quantity=0,reserved=0 WHERE tenant_id=1 AND sku_id=1",
        [],
    ))
    .await
    .unwrap();
    crate::test_log_capture::clear();
    let handled = crate::endpoints::checkout_payment_endpoint::reconcile_unresolved(&state)
        .await
        .unwrap();
    let captured_logs = crate::test_log_capture::contents();
    assert_eq!(handled, 1, "{captured_logs}");
    assert!(captured_logs.contains(&format!(
        "WARN payment {payment_id} captured after the window; no refund attempted, manual follow-up"
    )));
    assert!(captured_logs.contains(&format!(
        "ERROR late_capture_unfulfillable: stock unavailable, no refund attempted, manual follow-up, purchase_id={purchase_id}"
    )));
    let refund_count: i64 = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT count(*) AS count FROM payment_transaction WHERE payment_id=$1 AND operation='refund'",
            [payment_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "count")
        .unwrap();
    assert_eq!(refund_count, 0);
    cleanup(root, db, schema).await.unwrap();
}
