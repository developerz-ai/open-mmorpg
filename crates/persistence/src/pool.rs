//! Async connection pool for the durable store, with a health check, bounded
//! connect retries, and the migration runner.
//!
//! This is control-plane setup, not the deterministic sim path, so a
//! `tokio::time` backoff between connect attempts is fine here.

use crate::config::DbConfig;
use crate::error::map_sqlx;
use omm_errors::{CoreError, CoreResult};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Build a pool, retrying transient connect failures with linear backoff.
///
/// Each attempt is bounded by [`DbConfig::connect_timeout`]; after
/// [`DbConfig::connect_retries`] failures the last error is logged and the call
/// returns [`CoreError::Internal`] (detail is never returned to the caller).
///
/// # Errors
/// [`CoreError::Internal`] if every attempt fails or times out.
pub async fn connect(cfg: &DbConfig) -> CoreResult<PgPool> {
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        let building = PgPoolOptions::new()
            .max_connections(cfg.max_connections)
            .acquire_timeout(cfg.acquire_timeout)
            .connect(cfg.url());

        match tokio::time::timeout(cfg.connect_timeout, building).await {
            Ok(Ok(pool)) => return Ok(pool),
            Ok(Err(err)) if attempt >= cfg.connect_retries => {
                tracing::error!(error = %err, attempts = attempt, "pool connect failed");
                return Err(CoreError::Internal);
            }
            Err(_elapsed) if attempt >= cfg.connect_retries => {
                tracing::error!(attempts = attempt, "pool connect timed out");
                return Err(CoreError::Internal);
            }
            Ok(Err(err)) => {
                tracing::warn!(error = %err, attempt, "pool connect retrying");
                backoff(attempt).await;
            }
            Err(_elapsed) => {
                tracing::warn!(attempt, "pool connect timed out, retrying");
                backoff(attempt).await;
            }
        }
    }
}

async fn backoff(attempt: u32) {
    tokio::time::sleep(Duration::from_millis(200 * u64::from(attempt))).await;
}

/// Round-trip a trivial query to prove the pool is live.
///
/// # Errors
/// Propagates a mapped [`CoreError`] if the query fails.
pub async fn health_check(pool: &PgPool) -> CoreResult<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(map_sqlx)?;
    Ok(())
}

/// Apply all embedded, forward-only migrations. Idempotent: already-applied
/// versions are skipped; an edited applied migration is a hard error (drift).
///
/// # Errors
/// [`CoreError::Internal`] if a migration fails to apply (detail is logged).
pub async fn run_migrations(pool: &PgPool) -> CoreResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "migration failed");
            CoreError::Internal
        })?;
    Ok(())
}
