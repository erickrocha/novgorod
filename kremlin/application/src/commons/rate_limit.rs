//! Fixed-window, in-process rate limiter for public routes that cost a carrier call (NOV-35).
//! One window per client key; state lives in the process, so a multi-instance deployment limits per instance.
use axum::http::HeaderMap;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_secs(60);
const DEFAULT_LIMIT: u32 = 10;
const MAX_KEYS: usize = 10_000;

pub struct RateLimiter {
    hits: Mutex<HashMap<String, (Instant, u32)>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self { hits: Mutex::new(HashMap::new()) }
    }

    /// `Ok` while `key` is within `limit` requests per window; `Err(seconds)` is the wait until the window ends.
    pub fn check_at(&self, key: &str, limit: u32, now: Instant) -> Result<(), u64> {
        let mut hits = self.hits.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if hits.len() >= MAX_KEYS {
            hits.retain(|_, (start, _)| now.duration_since(*start) < WINDOW);
        }
        if !hits.contains_key(key) && hits.len() >= MAX_KEYS {
            let oldest_key = hits
                .iter()
                .min_by_key(|(_, (start, _))| *start)
                .map(|(client, _)| client.clone());
            if let Some(oldest_key) = oldest_key {
                hits.remove(&oldest_key);
            }
        }
        let entry = hits.entry(key.to_owned()).or_insert((now, 0));
        if now.duration_since(entry.0) >= WINDOW {
            *entry = (now, 0);
        }
        if entry.1 >= limit {
            return Err(WINDOW.saturating_sub(now.duration_since(entry.0)).as_secs().max(1));
        }
        entry.1 += 1;
        Ok(())
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Requests per minute allowed to the shipping estimate (`ESTIMATE_RATE_LIMIT_PER_MINUTE`, default 10).
pub fn estimate_limit() -> u32 {
    std::env::var("ESTIMATE_RATE_LIMIT_PER_MINUTE")
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .filter(|limit| *limit > 0)
        .unwrap_or(DEFAULT_LIMIT)
}

/// Checks the estimate limit for the caller; `Err(seconds)` when it is exceeded.
pub fn check_estimate(headers: &HeaderMap, peer: Option<SocketAddr>) -> Result<(), u64> {
    static LIMITER: OnceLock<RateLimiter> = OnceLock::new();
    LIMITER
        .get_or_init(RateLimiter::new)
        .check_at(&client_key(headers, peer), estimate_limit(), Instant::now())
}

/// The client behind a reverse proxy is read from the header named by `CLIENT_IP_HEADER` (for example
/// `x-forwarded-for`, first value); without it the peer address is used. Set it only when every request
/// passes through a proxy that overwrites that header, or a client could pick its own key.
fn client_key(headers: &HeaderMap, peer: Option<SocketAddr>) -> String {
    client_key_with(headers, peer, std::env::var("CLIENT_IP_HEADER").ok().as_deref())
}

fn client_key_with(headers: &HeaderMap, peer: Option<SocketAddr>, header_name: Option<&str>) -> String {
    let forwarded = header_name
        .and_then(|name| headers.get(name.trim().to_ascii_lowercase()))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(|first| first.trim().to_owned())
        .filter(|value| !value.is_empty() && value.len() <= 64);
    forwarded
        .or_else(|| peer.map(|address| address.ip().to_string()))
        .unwrap_or_else(|| "unknown".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_the_limit_then_refuses_until_the_window_ends() {
        let limiter = RateLimiter::new();
        let start = Instant::now();
        for _ in 0..10 {
            assert!(limiter.check_at("a", 10, start).is_ok());
        }
        let wait = limiter.check_at("a", 10, start + Duration::from_secs(20)).unwrap_err();
        assert_eq!(wait, 40);
        assert!(limiter.check_at("b", 10, start).is_ok(), "other clients are independent");
        assert!(limiter.check_at("a", 10, start + Duration::from_secs(60)).is_ok(), "new window");
    }

    #[test]
    fn evicts_the_oldest_live_key_when_client_capacity_is_full() {
        let limiter = RateLimiter::new();
        let start = Instant::now();
        for index in 0..MAX_KEYS {
            let started_at = start + Duration::from_millis(index as u64);
            assert!(limiter.check_at(&format!("client-{index}"), 10, started_at).is_ok());
        }
        for _ in 1..10 {
            assert!(limiter.check_at("client-0", 10, start + Duration::from_secs(10)).is_ok());
        }
        assert_eq!(limiter.check_at("client-0", 10, start + Duration::from_secs(10)), Err(50));

        assert_eq!(limiter.hits.lock().unwrap().len(), MAX_KEYS);
        assert!(limiter.check_at("new-client", 10, start + Duration::from_secs(11)).is_ok());
        {
            let hits = limiter.hits.lock().unwrap();
            assert!(!hits.contains_key("client-0"));
            assert!(hits.contains_key("new-client"));
            assert_eq!(hits.len(), MAX_KEYS);
        }
        assert!(limiter.check_at("client-0", 10, start + Duration::from_secs(11)).is_ok());
        for _ in 1..10 {
            assert!(limiter.check_at("client-0", 10, start + Duration::from_secs(11)).is_ok());
        }
        assert_eq!(limiter.check_at("client-0", 10, start + Duration::from_secs(11)), Err(60));
        assert_eq!(limiter.hits.lock().unwrap().len(), MAX_KEYS);
        let hits = limiter.hits.lock().unwrap();
        assert!(!hits.contains_key("client-1"));
        assert!(hits.contains_key("client-0"));
        assert!(hits.contains_key("new-client"));
    }

    #[test]
    fn client_key_prefers_the_configured_header_and_falls_back_to_the_peer() {
        let peer: SocketAddr = "203.0.113.9:4000".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "198.51.100.7, 10.0.0.1".parse().unwrap());
        // header configured: its first value wins over the peer
        assert_eq!(client_key_with(&headers, Some(peer), Some("X-Forwarded-For")), "198.51.100.7");
        // header configured but absent or empty: the peer
        assert_eq!(client_key_with(&HeaderMap::new(), Some(peer), Some("x-forwarded-for")), "203.0.113.9");
        headers.insert("x-forwarded-for", "".parse().unwrap());
        assert_eq!(client_key_with(&headers, Some(peer), Some("x-forwarded-for")), "203.0.113.9");
        // nothing configured: the peer, and "unknown" without a peer
        assert_eq!(client_key_with(&headers, Some(peer), None), "203.0.113.9");
        assert_eq!(client_key_with(&HeaderMap::new(), None, None), "unknown");
    }
}
