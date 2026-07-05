//! World service — the cross-shard read/social plane: **chat**, **guilds**, the
//! **auction house** browse view, the **armory** projection, and the **world
//! feed** (a GTA-inspired stream of notable world events).
//!
//! It is not on any shard's hot path; it aggregates across shards and serves the
//! reads the operator web consumes, so it is HTTP/axum. Crucially it is a *read*
//! plane: the auction house's authoritative value moves execute transactionally
//! via `omm-persistence` (economy plumbing) — worldsvc coordinates and projects,
//! it never owns the write (docs/architecture/04-data-and-consistency.md).

mod feed;
mod market;
mod routes;
mod social;
mod state;

use std::net::SocketAddr;

use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    let state = AppState::new();
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("worldsvc listening on {addr}");
    axum::serve(listener, routes::app(state)).await?;
    Ok(())
}
