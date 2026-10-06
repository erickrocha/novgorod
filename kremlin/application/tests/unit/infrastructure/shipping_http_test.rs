use crate::{
    AppState,
    commons::{i18n::Locale, tenant_context::TenantContext},
    endpoints::checkout_quote_endpoint::quote,
    infrastructure::{
        correios::Correios,
        shipping_credentials::{CorreiosCredentials, ShippingKeyRing},
    },
};
use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    routing::post,
};
use business::{
    domain::{enums::Role, user::User},
    gateway::shipping_provider_gateway::ShippingProviderGateway,
    sea_orm::{ConnectOptions, ConnectionTrait, Database, DbBackend, DbConn, Statement},
};
use migration::{Migrator, MigratorTrait};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tower::ServiceExt;

const AUTH_FAILURE: usize = 1;
const MALFORMED_RESPONSE: usize = 2;
const TIMEOUT: usize = 3;
const PROVIDER_OUTAGE: usize = 4;
const SECRET_SENTINELS: [&str; 3] = [
    "correios-credential-sentinel",
    "correios-token-sentinel",
    "correios-body-sentinel",
];
static CAPTURED_LOGS: Mutex<Vec<String>> = Mutex::new(Vec::new());

struct CaptureLogger;

impl log::Log for CaptureLogger {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        CAPTURED_LOGS
            .lock()
            .unwrap()
            .push(record.args().to_string());
    }

    fn flush(&self) {}
}

static CAPTURE_LOGGER: CaptureLogger = CaptureLogger;

#[derive(Default)]
struct CorreiosDouble {
    mode: AtomicUsize,
    auth_calls: AtomicUsize,
    quote_calls: AtomicUsize,
}

async fn token(
    State(double): State<Arc<CorreiosDouble>>,
    headers: HeaderMap,
    Json(_body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    double.auth_calls.fetch_add(1, Ordering::SeqCst);
    if headers.get("authorization").is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"missing auth"})),
        );
    }
    if double.mode.load(Ordering::SeqCst) == AUTH_FAILURE {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"secret":"correios-body-sentinel"})),
        );
    }
    (
        StatusCode::OK,
        Json(json!({
            "token":"correios-token-sentinel",
            "expiraEm":(chrono::Utc::now()+chrono::Duration::hours(1)).to_rfc3339()
        })),
    )
}

async fn price(
    State(double): State<Arc<CorreiosDouble>>,
    Json(_body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    double.quote_calls.fetch_add(1, Ordering::SeqCst);
    match double.mode.load(Ordering::SeqCst) {
        MALFORMED_RESPONSE => (
            StatusCode::OK,
            Json(json!({"secret":"correios-body-sentinel"})),
        ),
        TIMEOUT => {
            tokio::time::sleep(Duration::from_millis(8_200)).await;
            (StatusCode::OK, Json(json!([])))
        }
        PROVIDER_OUTAGE => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"secret":"correios-body-sentinel"})),
        ),
        _ => (
            StatusCode::OK,
            Json(json!([{"coProduto":"03220","pcFinal":"24,50"}])),
        ),
    }
}

async fn deadline() -> Json<Value> {
    Json(json!([{"coProduto":"03220","prazoEntrega":2}]))
}

async fn start_double(double: Arc<CorreiosDouble>) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/token/v1/autentica/cartaopostagem", post(token))
        .route("/preco/v1/nacional", post(price))
        .route("/prazo/v1/nacional", post(deadline))
        .with_state(double);
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (base_url, task)
}

async fn database() -> anyhow::Result<(DbConn, DbConn, String)> {
    let url = std::env::var("KREMLIN_TEST_DATABASE_URL")?;
    anyhow::ensure!(
        url.split('?').next().unwrap_or_default().ends_with("_test"),
        "test database name must end in _test"
    );
    let root = Database::connect(url.clone()).await?;
    let schema = format!("shipping_http_test_{}", uuid::Uuid::new_v4().simple());
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
    anyhow::ensure!(schema.starts_with("shipping_http_test_"));
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
INSERT INTO tenant (id, uuid, business_name, tax_id, postal_code, created_at, updated_at)
VALUES (1, gen_random_uuid(), 'Seller', '12345678000100', '01001000', now(), now());
INSERT INTO product (id, uuid, tenant_id, name, slug, active, ncm, origem_mercadoria, created_at, updated_at)
VALUES (1, gen_random_uuid(), 1, 'Product', 'product', true, '12345678', 0, now(), now());
INSERT INTO sku (id, uuid, tenant_id, product_id, code, variant_key, price_cents, active, weight_g, length_mm, width_mm, height_mm, created_at, updated_at)
VALUES (1, gen_random_uuid(), 1, 1, 'SKU-1', 'default', 5000, true, 500, 131, 91, 21, now(), now());
INSERT INTO sku_stock (uuid, tenant_id, sku_id, quantity, reserved, created_at, updated_at)
VALUES (gen_random_uuid(), 1, 1, 10, 0, now(), now());
INSERT INTO shipping_rate (uuid, tenant_id, uf, price_cents, created_at, updated_at)
VALUES (gen_random_uuid(), 1, 'SP', 1500, now(), now());
"#,
    )
    .await?;
    Ok(())
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

fn router(state: AppState) -> Router {
    Router::new()
        .route("/checkout/quotes", post(quote))
        .with_state(state)
}

async fn quote_request(app: Router) -> anyhow::Result<(StatusCode, String)> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/checkout/quotes")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "items":[{"skuId":1,"quantity":1}],
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
            })
            .to_string(),
        ))?;
    request.extensions_mut().insert(user());
    request.extensions_mut().insert(Locale::En);
    request.extensions_mut().insert(TenantContext::Marketplace);
    let response = app.oneshot(request).await?;
    let status = response.status();
    let body = String::from_utf8(to_bytes(response.into_body(), usize::MAX).await?.to_vec())?;
    Ok((status, body))
}

fn state(
    db: DbConn,
    provider: Arc<dyn ShippingProviderGateway>,
    keys: Arc<ShippingKeyRing>,
) -> AppState {
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
        shipping: provider,
        shipping_keys: keys,
        payment_keys: Arc::new(
            crate::infrastructure::payment_credentials::PaymentKeyRing::default(),
        ),
        gateway_token: None,
    }
}

async fn execute_cases(db: DbConn) -> anyhow::Result<()> {
    log::set_logger(&CAPTURE_LOGGER)
        .map_err(|_| anyhow::anyhow!("test logger already installed"))?;
    log::set_max_level(log::LevelFilter::Trace);
    let keys = Arc::new(ShippingKeyRing::for_test());
    let double = Arc::new(CorreiosDouble::default());
    let (base_url, task) = start_double(double.clone()).await;

    let provider = Arc::new(
        Correios::with_base_url_for_test(db.clone(), keys.clone(), &base_url)
            .map_err(anyhow::Error::msg)?,
    );
    let fixed_app = router(state(db.clone(), provider.clone(), keys.clone()));
    let (status, body) = quote_request(fixed_app).await?;
    anyhow::ensure!(
        status == StatusCode::CREATED,
        "fixed quote returned {status}: {body}"
    );
    let fixed: Value = serde_json::from_str(&body)?;
    anyhow::ensure!(fixed["sellers"][0]["shipping"]["mode"] == "fixed");
    anyhow::ensure!(fixed["sellers"][0]["shipping"]["selectedOption"]["priceCents"] == 1500);
    anyhow::ensure!(double.auth_calls.load(Ordering::SeqCst) == 0);

    let credentials = CorreiosCredentials {
        username: "user-correios".into(),
        api_access_code: SECRET_SENTINELS[0].into(),
        posting_card: "card-correios".into(),
        contract: "contract-correios".into(),
        regional_identifier: "72".into(),
    };
    let (key_version, encrypted) = keys.encrypt(1, &credentials)?;
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO tenant_shipping_settings (tenant_id, configuration, credentials, key_version, credential_version) VALUES ($1, $2, $3, $4, 1)",
        [
            1.into(),
            json!({"mode":"correios","originCep":"01001000","services":[{"code":"03220","name":"SEDEX"}]}).into(),
            encrypted.into(),
            key_version.into(),
        ],
    ))
    .await?;

    let provider = Arc::new(
        Correios::with_base_url_for_test(db.clone(), keys.clone(), &base_url)
            .map_err(anyhow::Error::msg)?,
    );
    let app = router(state(db.clone(), provider, keys.clone()));
    let (status, body) = quote_request(app.clone()).await?;
    anyhow::ensure!(
        status == StatusCode::CREATED,
        "Correios quote returned {status}: {body}"
    );
    let correios: Value = serde_json::from_str(&body)?;
    anyhow::ensure!(correios["sellers"][0]["shipping"]["mode"] == "correios");
    anyhow::ensure!(correios["sellers"][0]["shipping"]["selectedOption"]["serviceCode"] == "03220");
    anyhow::ensure!(correios["sellers"][0]["shipping"]["selectedOption"]["priceCents"] == 2450);
    anyhow::ensure!(double.auth_calls.load(Ordering::SeqCst) == 1);

    CAPTURED_LOGS.lock().unwrap().clear();
    for mode in [AUTH_FAILURE, MALFORMED_RESPONSE, TIMEOUT, PROVIDER_OUTAGE] {
        double.mode.store(mode, Ordering::SeqCst);
        let provider = Arc::new(
            Correios::with_base_url_for_test(db.clone(), keys.clone(), &base_url)
                .map_err(anyhow::Error::msg)?,
        );
        let app = router(state(db.clone(), provider, keys.clone()));
        let (status, body) = quote_request(app).await?;
        anyhow::ensure!(
            status == StatusCode::SERVICE_UNAVAILABLE,
            "provider failure mode {mode} returned {status}: {body}"
        );
        for sentinel in SECRET_SENTINELS {
            anyhow::ensure!(!body.contains(sentinel), "HTTP response exposed {sentinel}");
        }
    }
    let logs = CAPTURED_LOGS.lock().unwrap().join("\n");
    for sentinel in SECRET_SENTINELS {
        anyhow::ensure!(
            !logs.contains(sentinel),
            "application logs exposed {sentinel}"
        );
    }

    task.abort();
    Ok(())
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL via KREMLIN_TEST_DATABASE_URL"]
async fn quote_http_covers_fixed_correios_and_sanitized_provider_failures() {
    let (root, db, schema) = database().await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    fixtures(&db).await.unwrap();
    let result = execute_cases(db.clone()).await;
    cleanup(root, db, schema).await.unwrap();
    result.unwrap();
}
