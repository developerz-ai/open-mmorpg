//! Shared gateway state.
//!
//! Holds the collaborators every handler needs — session signer, shard router,
//! rate limiter, credential verifier, and live realm status — behind `Arc`s so
//! the axum `State` clone per request is cheap.

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use crate::auth::CredentialVerifier;
use crate::ratelimit::KeyedRateLimiter;
use crate::routing::ShardRouter;
use crate::session::SessionSigner;

/// Live realm status, shared and cheaply readable by the public status route.
///
/// Population is an atomic so a presence feed can update it without a lock; it
/// starts at zero and is wired live once shard presence lands.
#[derive(Debug)]
pub struct RealmState {
    /// Public realm display name.
    pub name: String,
    /// Advertised capacity.
    pub capacity: u32,
    /// Current player population.
    pub population: AtomicU64,
}

impl RealmState {
    /// Build realm state starting at zero population.
    #[must_use]
    pub fn new(name: impl Into<String>, capacity: u32) -> Self {
        Self {
            name: name.into(),
            capacity,
            population: AtomicU64::new(0),
        }
    }
}

/// Shared gateway state injected into every handler.
///
/// Generic over the credential verifier `V` so the DB-backed implementation can
/// replace [`crate::auth::DevVerifier`] with no `dyn` and no handler change. The
/// shard router stays `dyn` (it is synchronous and object-safe).
pub struct AppState<V: CredentialVerifier> {
    pub(crate) signer: Arc<SessionSigner>,
    pub(crate) router: Arc<dyn ShardRouter>,
    pub(crate) limiter: Arc<KeyedRateLimiter>,
    pub(crate) verifier: Arc<V>,
    pub(crate) realm: Arc<RealmState>,
    pub(crate) token_ttl_secs: u64,
}

// Manual `Clone` so `V` need not be `Clone` (everything is behind `Arc`).
impl<V: CredentialVerifier> Clone for AppState<V> {
    fn clone(&self) -> Self {
        Self {
            signer: Arc::clone(&self.signer),
            router: Arc::clone(&self.router),
            limiter: Arc::clone(&self.limiter),
            verifier: Arc::clone(&self.verifier),
            realm: Arc::clone(&self.realm),
            token_ttl_secs: self.token_ttl_secs,
        }
    }
}

impl<V: CredentialVerifier> AppState<V> {
    /// Assemble gateway state from its collaborators.
    #[must_use]
    pub fn new(
        signer: Arc<SessionSigner>,
        router: Arc<dyn ShardRouter>,
        limiter: Arc<KeyedRateLimiter>,
        verifier: Arc<V>,
        realm: Arc<RealmState>,
        token_ttl_secs: u64,
    ) -> Self {
        Self {
            signer,
            router,
            limiter,
            verifier,
            realm,
            token_ttl_secs,
        }
    }
}
