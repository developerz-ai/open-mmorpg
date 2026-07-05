//! HTTP surface — the read/social plane the operator web consumes. Handlers are
//! thin adapters over the projections in [`crate::state`]; all domain logic and
//! its tests live in the projection modules. Errors carry the stable
//! [`omm_errors::ClientCode`] as JSON, never internal detail.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use omm_errors::CoreError;
use serde::{Deserialize, Serialize};

use crate::feed::FeedEntry;
use crate::market::{CharacterSummary, Listing, ListingId};
use crate::social::{ChatMessage, Guild, GuildId};
use crate::state::AppState;

/// A wire error: the stable client code plus a safe, human-readable message.
#[derive(Debug, Serialize)]
struct ApiError {
    code: u16,
    message: String,
}

/// Map a [`CoreError`] to an HTTP response with its stable client code. We match
/// on the *numeric* code (part of the wire contract) rather than the enum, so a
/// new `ClientCode` variant can't silently break this mapping — unknown codes
/// fall through to 500.
fn api_error(err: &CoreError) -> Response {
    let code = err.code().as_u16();
    let status = match code {
        1000 => StatusCode::BAD_REQUEST,
        1001 => StatusCode::UNAUTHORIZED,
        1002 => StatusCode::FORBIDDEN,
        1003 => StatusCode::NOT_FOUND,
        1004 => StatusCode::CONFLICT,
        1005 => StatusCode::TOO_MANY_REQUESTS,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (
        status,
        Json(ApiError {
            code,
            message: err.to_string(),
        }),
    )
        .into_response()
}

/// Build the worldsvc router. Kept separate from `main` so tests exercise the
/// exact wiring the server runs. `GET`s are the web-facing read plane; `POST`s
/// (and the `DELETE`) are the internal ingest shards push projections through.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/world/feed", get(feed).post(publish_feed))
        .route("/chat/{channel}", get(chat_recent).post(chat_post))
        .route("/guild", post(guild_create))
        .route("/guild/{id}", get(guild_get))
        .route("/guild/{id}/join", post(guild_join))
        .route("/armory", get(armory_search).post(armory_upsert))
        .route("/armory/{id}", get(armory_get))
        .route("/auction", get(auction_browse).post(auction_list))
        .route("/auction/{id}", axum::routing::delete(auction_remove))
        .with_state(state)
}

/// `?limit=` with a sane default, shared by feed/chat.
#[derive(Debug, Deserialize)]
struct LimitQuery {
    limit: Option<usize>,
}

fn limit_or(q: &LimitQuery, default: usize) -> usize {
    q.limit.unwrap_or(default).min(200)
}

async fn feed(State(s): State<AppState>, Query(q): Query<LimitQuery>) -> Json<Vec<FeedEntry>> {
    Json(s.feed.read().await.recent(limit_or(&q, 50)))
}

async fn chat_recent(
    State(s): State<AppState>,
    Path(channel): Path<String>,
    Query(q): Query<LimitQuery>,
) -> Json<Vec<ChatMessage>> {
    Json(s.chat.read().await.recent(&channel, limit_or(&q, 50)))
}

#[derive(Debug, Deserialize)]
struct PostChat {
    sender: u64,
    body: String,
    tick: u64,
}

async fn chat_post(
    State(s): State<AppState>,
    Path(channel): Path<String>,
    Json(m): Json<PostChat>,
) -> StatusCode {
    s.chat.write().await.post(
        &channel,
        ChatMessage {
            sender: m.sender,
            body: m.body,
            tick: m.tick,
        },
    );
    StatusCode::ACCEPTED
}

#[derive(Debug, Deserialize)]
struct CreateGuild {
    name: String,
    founder: u64,
}

#[derive(Debug, Serialize)]
struct GuildCreated {
    id: u64,
}

async fn guild_create(State(s): State<AppState>, Json(c): Json<CreateGuild>) -> Response {
    match s.guilds.write().await.create(&c.name, c.founder) {
        Ok(id) => (StatusCode::CREATED, Json(GuildCreated { id: id.0 })).into_response(),
        Err(e) => api_error(&e),
    }
}

async fn guild_get(State(s): State<AppState>, Path(id): Path<u64>) -> Response {
    match s.guilds.read().await.get(GuildId(id)).cloned() {
        Some(g) => Json::<Guild>(g).into_response(),
        None => api_error(&CoreError::NotFound(format!("guild {id}"))),
    }
}

#[derive(Debug, Deserialize)]
struct JoinGuild {
    member: u64,
}

async fn guild_join(
    State(s): State<AppState>,
    Path(id): Path<u64>,
    Json(j): Json<JoinGuild>,
) -> Response {
    match s.guilds.write().await.join(GuildId(id), j.member) {
        Ok(()) => StatusCode::ACCEPTED.into_response(),
        Err(e) => api_error(&e),
    }
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

async fn armory_search(State(s): State<AppState>, Query(q): Query<SearchQuery>) -> Response {
    let hits = s.armory.read().await.search(q.q.as_deref().unwrap_or(""));
    Json(hits).into_response()
}

async fn armory_get(State(s): State<AppState>, Path(id): Path<u64>) -> Response {
    match s.armory.read().await.get(id).cloned() {
        Some(c) => Json(c).into_response(),
        None => api_error(&CoreError::NotFound(format!("character {id}"))),
    }
}

#[derive(Debug, Deserialize)]
struct AuctionQuery {
    item: Option<String>,
}

async fn auction_browse(
    State(s): State<AppState>,
    Query(q): Query<AuctionQuery>,
) -> Json<Vec<Listing>> {
    Json(s.auctions.read().await.browse(q.item.as_deref()))
}

// ── Ingest: shards push projection updates here (internal plane) ──────────────

async fn publish_feed(State(s): State<AppState>, Json(e): Json<FeedEntry>) -> StatusCode {
    s.feed.write().await.publish(e);
    StatusCode::ACCEPTED
}

async fn armory_upsert(State(s): State<AppState>, Json(c): Json<CharacterSummary>) -> StatusCode {
    s.armory.write().await.upsert(c);
    StatusCode::ACCEPTED
}

#[derive(Debug, Deserialize)]
struct ListReq {
    item_def: String,
    seller: u64,
    price: u64,
    quantity: u32,
}

#[derive(Debug, Serialize)]
struct Listed {
    id: u64,
}

async fn auction_list(State(s): State<AppState>, Json(r): Json<ListReq>) -> Response {
    match s
        .auctions
        .write()
        .await
        .list(&r.item_def, r.seller, r.price, r.quantity)
    {
        Ok(id) => (StatusCode::CREATED, Json(Listed { id: id.0 })).into_response(),
        Err(e) => api_error(&e),
    }
}

async fn auction_remove(State(s): State<AppState>, Path(id): Path<u64>) -> Response {
    match s.auctions.write().await.remove(ListingId(id)) {
        Some(l) => Json(l).into_response(),
        None => api_error(&CoreError::NotFound(format!("listing {id}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_builds() {
        let _ = app(AppState::new());
    }

    #[test]
    fn limit_is_clamped() {
        assert_eq!(limit_or(&LimitQuery { limit: None }, 50), 50);
        assert_eq!(limit_or(&LimitQuery { limit: Some(9999) }, 50), 200);
    }
}
