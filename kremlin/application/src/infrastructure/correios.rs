use super::shipping_credentials::{CorreiosCredentials, ShippingKeyRing};
use business::{
    domain::shipping::{Parcel, ShippingOption, ShippingService},
    gateway::shipping_provider_gateway::{
        ShippingProviderError as Error, ShippingProviderGateway, ShippingRequest,
    },
    sea_orm::{ConnectionTrait, DbBackend, DbConn, Statement, prelude::async_trait::async_trait},
};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tokio::sync::{Mutex, Semaphore};

struct Token {
    value: String,
    expires_at: i64,
    credential_version: i64,
}
type TokenSlot = Arc<Mutex<Option<Token>>>;
pub struct Correios {
    db: DbConn,
    keys: Arc<ShippingKeyRing>,
    client: reqwest::Client,
    base_url: String,
    tokens: Mutex<BTreeMap<i64, TokenSlot>>,
    permits: Semaphore,
}
impl Correios {
    pub fn new(db: DbConn, keys: Arc<ShippingKeyRing>) -> Result<Self, &'static str> {
        let base = match std::env::var("CORREIOS_ENVIRONMENT").as_deref() {
            Ok("production") => "https://api.correios.com.br",
            Ok("homologation") | Err(_) => "https://apihom.correios.com.br",
            _ => return Err("invalid CORREIOS_ENVIRONMENT"),
        };
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .connect_timeout(Duration::from_secs(3))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "cannot initialize Correios client")?;
        Ok(Self {
            db,
            keys,
            client,
            base_url: base.into(),
            tokens: Mutex::new(BTreeMap::new()),
            permits: Semaphore::new(8),
        })
    }
    async fn slot(&self, tenant_id: i64) -> TokenSlot {
        let mut cache = self.tokens.lock().await;
        if cache.len() >= 1024 && !cache.contains_key(&tenant_id) {
            cache.pop_first();
        }
        cache
            .entry(tenant_id)
            .or_insert_with(|| Arc::new(Mutex::new(None)))
            .clone()
    }
    async fn token(
        &self,
        slot: &TokenSlot,
        version: i64,
        credentials: &CorreiosCredentials,
        rejected: Option<&str>,
    ) -> Result<String, Error> {
        // Coalesce concurrent authentication and avoid invalidating a newer refreshed token.
        let mut cached = slot.lock().await;
        if let Some(token) = cached.as_ref() {
            if token.credential_version == version
                && token.expires_at > Utc::now().timestamp() + 60
                && rejected != Some(token.value.as_str())
            {
                return Ok(token.value.clone());
            }
        }
        *cached = None;
        let regional = credentials
            .regional_identifier
            .parse::<u32>()
            .map_err(|_| Error::Credentials)?;
        let response = self.client.post(format!("{}/token/v1/autentica/cartaopostagem", self.base_url))
            .basic_auth(&credentials.username, Some(&credentials.api_access_code))
            .json(&json!({"numero": credentials.posting_card, "contrato": credentials.contract, "dr": regional}))
            .send().await.map_err(|_| Error::Unavailable)?;
        if !response.status().is_success() {
            return Err(if response.status().is_client_error() {
                Error::Credentials
            } else {
                Error::Unavailable
            });
        }
        let value = Self::body(response).await?;
        let token = value["token"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or(Error::InvalidResponse)?
            .to_owned();
        let expires = value["expiraEm"].as_str().ok_or(Error::InvalidResponse)?;
        let expires_at = expiration(expires)?;
        if expires_at <= Utc::now().timestamp() {
            return Err(Error::InvalidResponse);
        }
        *cached = Some(Token {
            value: token.clone(),
            expires_at,
            credential_version: version,
        });
        Ok(token)
    }
    async fn body(mut response: reqwest::Response) -> Result<Value, Error> {
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| Error::Unavailable)? {
            if bytes.len() + chunk.len() > 256 * 1024 {
                return Err(Error::InvalidResponse);
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes).map_err(|_| Error::InvalidResponse)
    }
    async fn post(
        &self,
        path: &str,
        body: &Value,
        token: &str,
    ) -> Result<(reqwest::StatusCode, Option<Value>), Error> {
        let response = self
            .client
            .post(format!("{}{path}", self.base_url))
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|_| Error::Unavailable)?;
        let status = response.status();
        if status.is_success() {
            Ok((status, Some(Self::body(response).await?)))
        } else {
            Ok((status, None))
        }
    }
    async fn query(
        &self,
        request: ShippingRequest,
        credentials: CorreiosCredentials,
        version: i64,
    ) -> Result<Vec<ShippingOption>, Error> {
        let p = &request.parcel;
        let l = Parcel::centimeters(p.length_mm).map_err(|_| Error::UnsupportedParcel)?;
        let w = Parcel::centimeters(p.width_mm).map_err(|_| Error::UnsupportedParcel)?;
        let h = Parcel::centimeters(p.height_mm).map_err(|_| Error::UnsupportedParcel)?;
        if p.weight_g <= 0
            || p.weight_g > 30_000
            || [l, w, h].iter().any(|d| *d > 100)
            || l + w + h > 200
        {
            return Err(Error::UnsupportedParcel);
        }
        let regional = credentials
            .regional_identifier
            .parse::<u32>()
            .map_err(|_| Error::Credentials)?;
        let products: Vec<_> = request.services.iter().map(|s| json!({
            "coProduto": s.code, "nuRequisicao": s.code, "nuContrato": credentials.contract, "nuDR": regional,
            "cepOrigem": request.origin_cep, "cepDestino": request.destination_cep,
            "psObjeto": p.weight_g.to_string(), "tpObjeto": "2", "comprimento": l.to_string(), "largura": w.to_string(), "altura": h.to_string()
        })).collect();
        let deadlines: Vec<_> = request.services.iter().map(|s| json!({"coProduto": s.code, "nuRequisicao": s.code, "cepOrigem": request.origin_cep, "cepDestino": request.destination_cep})).collect();
        let slot = self.slot(request.tenant_id).await;
        let mut token = self.token(&slot, version, &credentials, None).await?;
        let price_body = json!({"parametrosProduto": products});
        let time_body = json!({"parametrosPrazo": deadlines});
        for attempt in 0..2 {
            let (price, time) = tokio::try_join!(
                self.post("/preco/v1/nacional", &price_body, &token),
                self.post("/prazo/v1/nacional", &time_body, &token)
            )?;
            if price.0 == reqwest::StatusCode::UNAUTHORIZED
                || time.0 == reqwest::StatusCode::UNAUTHORIZED
            {
                if attempt == 1 {
                    return Err(Error::Credentials);
                }
                token = self
                    .token(&slot, version, &credentials, Some(&token))
                    .await?;
                continue;
            }
            for status in [price.0, time.0] {
                if status == reqwest::StatusCode::FORBIDDEN {
                    return Err(Error::Credentials);
                }
                if status == reqwest::StatusCode::BAD_REQUEST
                    || status == reqwest::StatusCode::UNPROCESSABLE_ENTITY
                {
                    return Err(Error::NoDelivery);
                }
                if !status.is_success() {
                    return Err(Error::Unavailable);
                }
            }
            return merge(
                &request.services,
                price.1.ok_or(Error::InvalidResponse)?,
                time.1.ok_or(Error::InvalidResponse)?,
            );
        }
        Err(Error::Credentials)
    }
    async fn quote_inner(&self, request: ShippingRequest) -> Result<Vec<ShippingOption>, Error> {
        let _permit = self
            .permits
            .acquire()
            .await
            .map_err(|_| Error::Unavailable)?;
        let row = self.db.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres,
            "SELECT credentials,key_version,credential_version FROM tenant_shipping_settings WHERE tenant_id=$1 AND version=$2",
            [request.tenant_id.into(), request.configuration_version.into()])).await.map_err(|_| Error::Unavailable)?.ok_or(Error::Credentials)?;
        let bytes: Vec<u8> = row
            .try_get("", "credentials")
            .map_err(|_| Error::Credentials)?;
        let key: String = row
            .try_get("", "key_version")
            .map_err(|_| Error::Credentials)?;
        let version: i64 = row
            .try_get("", "credential_version")
            .map_err(|_| Error::Credentials)?;
        let credentials = self
            .keys
            .decrypt(request.tenant_id, &key, &bytes)
            .map_err(|_| Error::Credentials)?;
        self.query(request, credentials, version).await
    }
}
#[async_trait]
impl ShippingProviderGateway for Correios {
    async fn quote(&self, request: ShippingRequest) -> Result<Vec<ShippingOption>, Error> {
        tokio::time::timeout(Duration::from_secs(25), self.quote_inner(request))
            .await
            .map_err(|_| Error::Unavailable)?
    }
}
fn expiration(value: &str) -> Result<i64, Error> {
    if let Ok(date) = DateTime::parse_from_rfc3339(value) {
        return Ok(date.timestamp());
    }
    let local = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
        .map_err(|_| Error::InvalidResponse)?;
    chrono::FixedOffset::west_opt(3 * 3600)
        .ok_or(Error::InvalidResponse)?
        .from_local_datetime(&local)
        .single()
        .map(|v| v.timestamp())
        .ok_or(Error::InvalidResponse)
}
fn cents(value: &str) -> Result<i64, Error> {
    let normalized = if value.contains(',') {
        let (whole, fraction) = value.split_once(',').ok_or(Error::InvalidResponse)?;
        if fraction.len() != 2 || value.matches(',').count() != 1 {
            return Err(Error::InvalidResponse);
        }
        let groups: Vec<_> = whole.split('.').collect();
        if groups.iter().enumerate().any(|(i, g)| {
            g.is_empty() || !g.bytes().all(|c| c.is_ascii_digit()) || (i > 0 && g.len() != 3)
        }) {
            return Err(Error::InvalidResponse);
        }
        format!("{}.{}", groups.concat(), fraction)
    } else {
        value.to_owned()
    };
    let (whole, fraction) = normalized.split_once('.').ok_or(Error::InvalidResponse)?;
    if whole.is_empty()
        || fraction.len() != 2
        || !whole
            .bytes()
            .chain(fraction.bytes())
            .all(|c| c.is_ascii_digit())
    {
        return Err(Error::InvalidResponse);
    }
    whole
        .parse::<i64>()
        .ok()
        .and_then(|v| v.checked_mul(100))
        .and_then(|v| v.checked_add(fraction.parse().ok()?))
        .ok_or(Error::InvalidResponse)
}
fn indexed(value: Value) -> Result<BTreeMap<String, Value>, Error> {
    let values = value.as_array().ok_or(Error::InvalidResponse)?;
    let mut result = BTreeMap::new();
    for item in values {
        let code = item["coProduto"].as_str().ok_or(Error::InvalidResponse)?;
        if result.insert(code.to_owned(), item.clone()).is_some() {
            return Err(Error::InvalidResponse);
        }
    }
    Ok(result)
}
fn merge(
    services: &[ShippingService],
    prices: Value,
    times: Value,
) -> Result<Vec<ShippingOption>, Error> {
    let prices = indexed(prices)?;
    let times = indexed(times)?;
    let mut options = Vec::new();
    for service in services {
        let (Some(price), Some(time)) = (prices.get(&service.code), times.get(&service.code))
        else {
            continue;
        };
        if [price, time].iter().any(|v| {
            v.get("txErro")
                .is_some_and(|v| !v.is_null() && v.as_str() != Some(""))
        }) {
            continue;
        }
        let price_cents = cents(price["pcFinal"].as_str().ok_or(Error::InvalidResponse)?)?;
        let days = time["prazoEntrega"]
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .ok_or(Error::InvalidResponse)?;
        options.push(ShippingOption {
            id: uuid::Uuid::new_v4().to_string(),
            provider: "correios".into(),
            service_code: service.code.clone(),
            service_name: service.name.clone(),
            price_cents,
            transit_days: Some(days),
        });
    }
    if options.is_empty() {
        return Err(Error::NoDelivery);
    }
    Ok(options)
}

#[cfg(test)]
mod tests {
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
        assert_eq!(body["parametrosProduto"][0]["comprimento"], "14");
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
}
