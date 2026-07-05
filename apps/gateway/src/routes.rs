//! HTTP control-plane routes and shared state.
//!
//! The gateway is an axum control surface, **not** the realtime hot path (that
//! is UDP in `apps/shard`). Endpoints: `POST /auth/login`, `POST /auth/verify`,
//! `GET /realm/status`, `GET /health`. Every error is a typed [`GatewayError`]
//! carrying a stable client code — no internal detail leaks.

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use omm_protocol::ZoneId;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;

use crate::auth::{CredentialVerifier, Credentials};
use crate::error::GatewayError;
use crate::routing::ShardHandle;
use crate::session::{Token, UnixSeconds};
use crate::state::AppState;

/// Build the gateway router over `state`.
pub fn router<V: CredentialVerifier>(state: AppState<V>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/realm/status", get(realm_status::<V>))
        .route("/auth/login", post(login::<V>))
        .route("/auth/verify", post(verify_token::<V>))
        .with_state(state)
}

/// Liveness probe.
async fn health() -> &'static str {
    "ok"
}

/// Public, read-only realm status — no auth, never exposes topology.
#[derive(Debug, Serialize)]
struct RealmStatus {
    name: String,
    online: bool,
    population: u64,
    capacity: u32,
}

async fn realm_status<V: CredentialVerifier>(State(st): State<AppState<V>>) -> Json<RealmStatus> {
    Json(RealmStatus {
        name: st.realm.name.clone(),
        online: true,
        population: st.realm.population.load(Ordering::Relaxed),
        capacity: st.realm.capacity,
    })
}

/// `POST /auth/login` body: credentials plus an optional home zone to route to.
#[derive(Debug, Deserialize)]
struct LoginRequest {
    #[serde(flatten)]
    credentials: Credentials,
    /// Zone the player is entering; defaults to the home zone when omitted.
    #[serde(default)]
    zone: Option<u64>,
}

/// `POST /auth/login` success body.
#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    shard: ShardHandle,
    expires_at: u64,
}

async fn login<V: CredentialVerifier>(
    State(st): State<AppState<V>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, GatewayError> {
    // Rate-limit per account handle first — blunts credential-stuffing floods
    // before doing any auth work.
    if !st.limiter.check(&body.credentials.username).await {
        return Err(GatewayError::RateLimited);
    }

    let account = st.verifier.verify(&body.credentials).await?;
    let zone = ZoneId::new(body.zone.unwrap_or(0));
    let shard = st.router.route(zone);

    let now = UnixSeconds::now();
    let token = st.signer.issue(account, now, st.token_ttl_secs)?;

    Ok(Json(LoginResponse {
        token: token.into_string(),
        shard,
        expires_at: now.0.saturating_add(st.token_ttl_secs),
    }))
}

/// `POST /auth/verify` body.
#[derive(Debug, Deserialize)]
struct VerifyRequest {
    token: String,
}

/// `POST /auth/verify` success body.
#[derive(Debug, Serialize)]
struct VerifyResponse {
    account: u64,
    expires_at: u64,
}

async fn verify_token<V: CredentialVerifier>(
    State(st): State<AppState<V>>,
    Json(body): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, GatewayError> {
    let claims = st
        .signer
        .verify(&Token::new(body.token), UnixSeconds::now())?;
    Ok(Json(VerifyResponse {
        account: claims.account.raw(),
        expires_at: claims.expires_at.0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::DevVerifier;
    use crate::ratelimit::{KeyedRateLimiter, RateConfig};
    use crate::routing::{HashRouter, ShardRouter};
    use crate::session::SessionSigner;
    use crate::state::RealmState;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt as _;
    use omm_protocol::ShardId;
    use std::sync::Arc;
    use tower::ServiceExt as _; // oneshot

    fn test_state() -> AppState<DevVerifier> {
        let router: Arc<dyn ShardRouter> =
            Arc::new(HashRouter::new(vec![ShardId::new(1), ShardId::new(2)]).unwrap());
        AppState::new(
            Arc::new(SessionSigner::new(b"unit-test-secret".to_vec())),
            router,
            Arc::new(KeyedRateLimiter::new(RateConfig {
                capacity: 3,
                refill_per_sec: 0.0,
            })),
            Arc::new(DevVerifier),
            Arc::new(RealmState::new("test-realm", 1000)),
            3600,
        )
    }

    async fn body_json(resp: axum::response::Response) -> serde_json::Value {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    fn post(path: &str, json: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&json).unwrap()))
            .unwrap()
    }

    #[tokio::test]
    async fn health_ok() {
        let resp = router(test_state())
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn realm_status_reports_population() {
        let state = test_state();
        state.realm.population.store(7, Ordering::Relaxed);
        let resp = router(state)
            .oneshot(
                Request::builder()
                    .uri("/realm/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert_eq!(json["population"], 7);
        assert_eq!(json["name"], "test-realm");
    }

    #[tokio::test]
    async fn login_issues_a_verifiable_token() {
        let state = test_state();
        let app = router(state.clone());
        let resp = app
            .oneshot(post(
                "/auth/login",
                serde_json::json!({ "username": "neo", "password": "trinity", "zone": 5 }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        let token = json["token"].as_str().unwrap().to_owned();
        assert!(json["shard"].as_str().unwrap().starts_with("shard-"));

        // The issued token verifies through the /auth/verify route.
        let resp = router(state)
            .oneshot(post("/auth/verify", serde_json::json!({ "token": token })))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn login_rejects_blank_credentials() {
        let resp = router(test_state())
            .oneshot(post(
                "/auth/login",
                serde_json::json!({ "username": "", "password": "" }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let json = body_json(resp).await;
        assert_eq!(json["code"], 1001);
    }

    #[tokio::test]
    async fn verify_rejects_forged_token() {
        let resp = router(test_state())
            .oneshot(post(
                "/auth/verify",
                serde_json::json!({ "token": "forged.token" }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn login_is_rate_limited() {
        // capacity 3, no refill → 4th login for the same account is 429.
        let state = test_state();
        let app = router(state);
        for _ in 0..3 {
            let resp = app
                .clone()
                .oneshot(post(
                    "/auth/login",
                    serde_json::json!({ "username": "spammer", "password": "x" }),
                ))
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
        let resp = app
            .oneshot(post(
                "/auth/login",
                serde_json::json!({ "username": "spammer", "password": "x" }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let json = body_json(resp).await;
        assert_eq!(json["code"], 1005);
    }
}
