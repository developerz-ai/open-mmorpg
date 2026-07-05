//! World service — the cross-shard plane: chat, guilds, auction house, and the
//! **world feed** (a GTA-inspired stream of notable world events).
//!
//! It is not on any shard's hot path; it aggregates and fans out across shards,
//! so it is HTTP/axum. The auction house's ownership changes are still executed
//! transactionally via `omm-persistence` — worldsvc coordinates, it does not own
//! the write (docs/architecture/04-data-and-consistency.md).

use std::net::SocketAddr;

use axum::{routing::get, Json, Router};
use serde::Serialize;

/// A single entry in the public world feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FeedEntry {
    kind: &'static str,
    message: &'static str,
}

/// The current world feed. Static seed in the scaffold; the live version pulls
/// recent cross-shard events from `omm-cache`.
fn world_feed() -> Vec<FeedEntry> {
    vec![FeedEntry {
        kind: "system",
        message: "The world awakens.",
    }]
}

fn app() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/world/feed", get(|| async { Json(world_feed()) }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("worldsvc listening on {addr}");
    axum::serve(listener, app()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feed_has_a_seed_entry() {
        let feed = world_feed();
        assert_eq!(feed.len(), 1);
        assert_eq!(feed[0].kind, "system");
    }

    #[test]
    fn router_builds() {
        let _ = app();
    }
}
