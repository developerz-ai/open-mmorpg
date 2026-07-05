//! Shared application state. Each projection sits behind its own async lock so a
//! chat write can't block an armory read. `tokio::sync` only — never
//! `std::sync::Mutex` on the request path.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::feed::WorldFeed;
use crate::market::{Armory, AuctionBoard};
use crate::social::{ChatHub, GuildRegistry};

/// The worldsvc read/social plane, cheaply cloneable (all `Arc`) for axum state.
#[derive(Clone)]
pub struct AppState {
    pub feed: Arc<RwLock<WorldFeed>>,
    pub chat: Arc<RwLock<ChatHub>>,
    pub guilds: Arc<RwLock<GuildRegistry>>,
    pub armory: Arc<RwLock<Armory>>,
    pub auctions: Arc<RwLock<AuctionBoard>>,
}

impl AppState {
    /// Fresh, empty projections.
    #[must_use]
    pub fn new() -> Self {
        Self {
            feed: Arc::new(RwLock::new(WorldFeed::default())),
            chat: Arc::new(RwLock::new(ChatHub::new(256))),
            guilds: Arc::new(RwLock::new(GuildRegistry::new())),
            armory: Arc::new(RwLock::new(Armory::new())),
            auctions: Arc::new(RwLock::new(AuctionBoard::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
