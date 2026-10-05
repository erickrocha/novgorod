use super::*;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use std::sync::atomic::{AtomicUsize, Ordering};

fn services() -> Vec<ShippingService> {
    vec![
        ShippingService {
            code: "03220".into(),
            name: "SEDEX".into(),
        },
        ShippingService {
            code: "03298".into(),
            name: "PAC".into(),
        },
    ]
}
fn request(id: i64) -> ShippingRequest {
    ShippingRequest {
        tenant_id: id,
        configuration_version: 1,
        origin_cep: "01001000".into(),
        destination_cep: "20040002".into(),
        parcel: Parcel {
            weight_g: 500,
            length_mm: 131,
            width_mm: 91,
            height_mm: 21,
        },
        services: services(),
    }
}
fn credentials() -> CorreiosCredentials {
    CorreiosCredentials {
        username: "u".into(),
        api_access_code: "secret".into(),
        posting_card: "card".into(),
        contract: "contract".into(),
        regional_identifier: "72".into(),
    }
}
#[derive(Default)]
struct Server {
    auth: AtomicUsize,
    prices: AtomicUsize,
    reject_first: bool,
    mode: usize,
    expected_parcel: Option<(&'static str, &'static str, &'static str, &'static str)>,
}
async fn auth(
    State(s): State<Arc<Server>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Json<Value> {
    assert_eq!(headers["authorization"], "Basic dTpzZWNyZXQ=");
    assert_eq!(body["numero"], "card");
    assert_eq!(body["dr"], 72);
    let n = s.auth.fetch_add(1, Ordering::SeqCst) + 1;
    Json(
        json!({"token": format!("token-{n}"), "expiraEm": (Utc::now()+chrono::Duration::hours(1)).to_rfc3339()}),
    )
}
async fn price(
    State(s): State<Arc<Server>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    s.prices.fetch_add(1, Ordering::SeqCst);
    if let Some((weight, length, width, height)) = s.expected_parcel {
        let parcel = &body["parametrosProduto"][0];
        assert_eq!(parcel["psObjeto"], weight);
        assert_eq!(parcel["comprimento"], length);
        assert_eq!(parcel["largura"], width);
        assert_eq!(parcel["altura"], height);
    } else {
        assert_eq!(body["parametrosProduto"][0]["comprimento"], "14");
    }
    assert_eq!(body["parametrosProduto"][0]["nuContrato"], "contract");
    assert_eq!(body["parametrosProduto"][0]["nuDR"], 72);
    if s.reject_first && headers["authorization"] == "Bearer token-1" {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"secret":"never expose"})),
        );
    }
    if s.mode == 1 {
        return (StatusCode::OK, Json(json!({"malformed":"secret"})));
    }
    if s.mode == 2 {
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    if s.mode == 3 {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"secret":"never expose"})),
        );
    }
    (
        StatusCode::OK,
        Json(
            json!([{"coProduto":"03298","pcFinal":"12,00"},{"coProduto":"03220","pcFinal":"24,50"}]),
        ),
    )
}
async fn time() -> Json<Value> {
    Json(
        json!([{"coProduto":"03220","prazoEntrega":2},{"coProduto":"03298","txErro":"unavailable"}]),
    )
}
async fn adapter(server: Arc<Server>) -> (Correios, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let router = Router::new()
        .route("/token/v1/autentica/cartaopostagem", post(auth))
        .route("/preco/v1/nacional", post(price))
        .route("/prazo/v1/nacional", post(time))
        .with_state(server);
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let db = business::sea_orm::MockDatabase::new(DbBackend::Postgres).into_connection();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(100))
        .build()
        .unwrap();
    (
        Correios {
            db,
            keys: Arc::new(ShippingKeyRing::default()),
            client,
            base_url,
            tokens: Mutex::new(BTreeMap::new()),
            permits: Semaphore::new(2),
        },
        task,
    )
}
#[tokio::test]
async fn http_authentication_refresh_cache_and_tenant_isolation() {
    let server = Arc::new(Server {
        reject_first: true,
        ..Default::default()
    });
    let (adapter, task) = adapter(server.clone()).await;
    let result = adapter.query(request(1), credentials(), 1).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].price_cents, 2450);
    assert_eq!(result[0].transit_days, Some(2));
    assert_eq!(server.auth.load(Ordering::SeqCst), 2);
    adapter.query(request(1), credentials(), 1).await.unwrap();
    assert_eq!(server.auth.load(Ordering::SeqCst), 2);
    adapter.query(request(2), credentials(), 1).await.unwrap();
    assert_eq!(server.auth.load(Ordering::SeqCst), 3);
    adapter.query(request(1), credentials(), 2).await.unwrap();
    assert_eq!(server.auth.load(Ordering::SeqCst), 4);
    // An expired cache entry is refreshed even with unchanged credentials.
    adapter
        .slot(1)
        .await
        .lock()
        .await
        .as_mut()
        .unwrap()
        .expires_at = 0;
    adapter.query(request(1), credentials(), 2).await.unwrap();
    assert_eq!(server.auth.load(Ordering::SeqCst), 5);
    task.abort();
}
#[tokio::test]
async fn http_malformed_timeout_and_outage_are_retryable_without_body_leaks() {
    for (mode, expected) in [
        (1, Error::InvalidResponse),
        (2, Error::Unavailable),
        (3, Error::Unavailable),
    ] {
        let (adapter, task) = adapter(Arc::new(Server {
            mode,
            ..Default::default()
        }))
        .await;
        let err = adapter
            .query(request(1), credentials(), 1)
            .await
            .unwrap_err();
        assert_eq!(err, expected);
        assert!(!format!("{err:?}").contains("secret"));
        task.abort();
    }
}

#[tokio::test]
async fn parcel_limits_accept_exact_maximum_and_reject_excess_before_http_calls() {
    let server = Arc::new(Server {
        expected_parcel: Some(("30000", "100", "99", "1")),
        ..Default::default()
    });
    let (adapter, task) = adapter(server.clone()).await;
    let mut at_limit = request(1);
    at_limit.parcel = Parcel {
        weight_g: 30_000,
        length_mm: 1_000,
        width_mm: 990,
        height_mm: 10,
    };
    assert!(adapter.query(at_limit, credentials(), 1).await.is_ok());
    let auth_calls = server.auth.load(Ordering::SeqCst);
    let price_calls = server.prices.load(Ordering::SeqCst);

    for parcel in [
        Parcel {
            weight_g: 30_001,
            length_mm: 100,
            width_mm: 100,
            height_mm: 100,
        },
        Parcel {
            weight_g: 500,
            length_mm: 1_001,
            width_mm: 100,
            height_mm: 100,
        },
        Parcel {
            weight_g: 500,
            length_mm: 1_000,
            width_mm: 1_000,
            height_mm: 10,
        },
    ] {
        let mut request = request(1);
        request.parcel = parcel;
        assert_eq!(
            adapter.query(request, credentials(), 1).await.unwrap_err(),
            Error::UnsupportedParcel
        );
    }

    assert_eq!(server.auth.load(Ordering::SeqCst), auth_calls);
    assert_eq!(server.prices.load(Ordering::SeqCst), price_calls);
    task.abort();
}

#[test]
fn parsing_and_partial_service_matching() {
    assert_eq!(cents("1.234,56").unwrap(), 123456);
    assert_eq!(cents("24.50").unwrap(), 2450);
    for invalid in [
        "-1,00",
        "NaN",
        "1,234",
        "1",
        "1.23.456,78",
        "9223372036854775807,00",
    ] {
        assert!(cents(invalid).is_err());
    }
    let results = merge(
        &services(),
        json!([{"coProduto":"03220","pcFinal":"10,10"},{"coProduto":"03298","pcFinal":"5,00"}]),
        json!([{"coProduto":"03220","prazoEntrega":1}]),
    )
    .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].service_code, "03220");
    assert_eq!(
        merge(&services(), json!([]), json!([])).unwrap_err(),
        Error::NoDelivery
    );
    assert!(
        merge(
            &services(),
            json!([{"coProduto":"03220"}]),
            json!([{"coProduto":"03220","prazoEntrega":1}])
        )
        .is_err()
    );
    assert_eq!(
        expiration("2026-09-26T12:00:00").unwrap(),
        expiration("2026-09-26T15:00:00Z").unwrap()
    );
}
