use sea_orm::{ConnectOptions, DbConn, DbErr};
use std::env;
use std::time::Duration;

/// `workout/application` and `workout/integration` are separate processes
/// (REST + gRPC) that each open their own pool against the same Postgres
/// instance, so both must size their half sensibly rather than relying on
/// driver defaults. Env-configurable so ops can tune per deployment without
/// a rebuild; `POOL_LABEL` in the log line makes it obvious which of the two
/// processes a given pool belongs to when tuning.
pub async fn connect(db_url: &str, pool_label: &str) -> Result<DbConn, DbErr> {
    let mut options = ConnectOptions::new(db_url.to_owned());
    options
        .max_connections(env_u32("DB_POOL_MAX_CONNECTIONS", 10))
        .min_connections(env_u32("DB_POOL_MIN_CONNECTIONS", 1))
        .connect_timeout(Duration::from_secs(env_u64("DB_POOL_CONNECT_TIMEOUT_SECS", 8)))
        .acquire_timeout(Duration::from_secs(env_u64("DB_POOL_ACQUIRE_TIMEOUT_SECS", 8)))
        .idle_timeout(Duration::from_secs(env_u64("DB_POOL_IDLE_TIMEOUT_SECS", 300)));

    log::info!(
        "[{}] Connecting to Postgres with max_connections={}, min_connections={}",
        pool_label,
        env_u32("DB_POOL_MAX_CONNECTIONS", 10),
        env_u32("DB_POOL_MIN_CONNECTIONS", 1),
    );

    sea_orm::Database::connect(options).await
}

fn env_u32(var: &str, default: u32) -> u32 {
    env::var(var).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}

fn env_u64(var: &str, default: u64) -> u64 {
    env::var(var).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}
