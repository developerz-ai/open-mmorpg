//! Wire protocol: the single source of truth for messages crossing the
//! server<->client boundary. Both the Rust client and the servers depend on
//! this crate, so a wire change is one edit in one place.
//!
//! **Client sends intent, never state** (server-authoritative, always). That is
//! why [`ClientMsg`] carries requests/intents and [`ServerMsg`] carries
//! authoritative results and snapshots.

pub mod ids;

use omm_errors::ClientCode;
use serde::{Deserialize, Serialize};

pub use ids::{AccountId, CharacterId, ItemId, ShardId, ZoneId};

/// The monotonic simulation tick. Snapshots and inputs are stamped with it so
/// the client can interpolate and the server can re-simulate for anti-cheat.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct Tick(pub u64);

impl Tick {
    /// The next tick.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

/// A position on the wire. Fixed to `f32` triples to match the client's ECS.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// A movement/action intent from the client. The server validates and applies
/// it; the client never asserts the resulting state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Intent {
    /// Requested move direction (unit-ish vector), applied under server rules.
    Move { dir: Vec3 },
    /// Use ability `id` on an optional target.
    UseAbility {
        id: u32,
        target: Option<CharacterId>,
    },
}

/// Messages the client is allowed to send. Intents and session control only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMsg {
    /// Open a session with a previously issued gateway token.
    Hello { token: String },
    /// A gameplay intent stamped with the client's last-seen tick.
    Input { tick: Tick, intent: Intent },
    /// Keep-alive.
    Ping { nonce: u64 },
}

/// Authoritative messages from a server to a client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerMsg {
    /// Session accepted; here is your character and the shard serving it.
    Welcome {
        character: CharacterId,
        shard: ShardId,
    },
    /// An authoritative world snapshot for `tick`.
    Snapshot { tick: Tick, self_pos: Vec3 },
    /// A rejected request, with a stable, wire-safe code.
    Rejected { code: u16 },
    /// Keep-alive reply.
    Pong { nonce: u64 },
}

impl ServerMsg {
    /// Build a rejection from a stable [`ClientCode`].
    #[must_use]
    pub const fn rejected(code: ClientCode) -> Self {
        Self::Rejected {
            code: code.as_u16(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_advances() {
        assert_eq!(Tick(1).next(), Tick(2));
    }

    #[test]
    fn client_msg_roundtrips_through_json() {
        let msg = ClientMsg::Input {
            tick: Tick(9),
            intent: Intent::Move {
                dir: Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ClientMsg = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn rejection_carries_stable_code() {
        assert_eq!(
            ServerMsg::rejected(ClientCode::Forbidden),
            ServerMsg::Rejected { code: 1002 }
        );
    }
}
