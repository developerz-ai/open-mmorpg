//! Gateway — the edge every client hits first.
//!
//! HTTP/axum control plane: auth, signed session tokens, shard routing, and
//! edge rate-limiting. It is **not** the realtime hot path (that is UDP, in
//! `apps/shard`). Logic lives in modules; `main` only wires config → state →
//! server. → `docs/architecture/02-server-topology.md`.

mod auth;
mod config;
mod error;
mod ratelimit;
mod routes;
mod routing;
mod session;
mod state;

use std::sync::Arc;

use crate::auth::DevVerifier;
use crate::config::GatewayConfig;
use crate::ratelimit::{KeyedRateLimiter, RateConfig};
use crate::routes::router;
use crate::routing::{HashRouter, ShardRouter};
use crate::session::SessionSigner;
use crate::state::{AppState, RealmState};

/// Assemble gateway state from resolved config. Fails if the shard set is empty.
fn build_state(cfg: &GatewayConfig) -> Result<AppState<DevVerifier>, Box<dyn std::error::Error>> {
    let shard_router: Arc<dyn ShardRouter> = Arc::new(HashRouter::new(cfg.shards.clone())?);
    let limiter = KeyedRateLimiter::new(RateConfig {
        capacity: cfg.login_burst,
        refill_per_sec: cfg.login_refill_per_sec,
    });
    Ok(AppState::new(
        Arc::new(SessionSigner::new(cfg.secret.clone())),
        shard_router,
        Arc::new(limiter),
        Arc::new(DevVerifier::new()),
        Arc::new(RealmState::new(cfg.realm_name.clone(), cfg.realm_capacity)),
        cfg.token_ttl_secs,
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    let cfg = GatewayConfig::from_env();
    let bind = cfg.bind;
    let app = router(build_state(&cfg)?);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!("gateway listening on {bind}");
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_state_from_default_config() {
        let cfg = GatewayConfig::default();
        assert!(build_state(&cfg).is_ok());
    }

    #[test]
    fn build_state_fails_without_shards() {
        let mut cfg = GatewayConfig::default();
        cfg.shards.clear();
        assert!(build_state(&cfg).is_err());
    }
}
