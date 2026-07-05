//! Gateway — the edge every client hits first.
//!
//! Responsibilities (growing): health, public realm status, then auth + session
//! tokens + routing to shards (docs/architecture/02-server-topology.md). It is
//! HTTP/axum because it is a control-plane surface, not the realtime hot path
//! (that is UDP, in `apps/shard`).

use std::net::SocketAddr;

use axum::{routing::get, Json, Router};
use serde::Serialize;

/// Public, read-only status of a realm — what the operator website and launcher
/// poll. No auth required; never exposes internal topology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RealmStatus {
    name: &'static str,
    online: bool,
    population: u32,
    capacity: u32,
}

/// Compute the realm status. Pure so it is trivially testable; the live version
/// reads population from presence in `omm-cache`.
fn realm_status() -> RealmStatus {
    RealmStatus {
        name: "open-mmorpg",
        online: true,
        population: 0,
        capacity: 100_000,
    }
}

/// Build the HTTP router. Kept separate from `main` so tests exercise the same
/// wiring the server runs.
fn app() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/realm/status", get(|| async { Json(realm_status()) }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("gateway listening on {addr}");
    axum::serve(listener, app()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realm_status_starts_online_and_empty() {
        let s = realm_status();
        assert!(s.online);
        assert_eq!(s.population, 0);
        assert!(s.capacity >= s.population);
    }

    #[test]
    fn router_builds() {
        // Constructing the router must not panic (route/handler type wiring).
        let _ = app();
    }
}
