//! Hook contexts — the immutable event payloads the core hands each hook.
//!
//! Each server event has one `*Ctx` carrying exactly what a module needs to
//! react, in typed ids (never a raw `u64` — the type system is the first
//! anti-cheat layer, and it does not stop at the module seam). Contexts are
//! read-only by design: this foundation exposes *observation* hooks. A later
//! slice can add a `&mut` world handle to contexts that must mutate; keeping the
//! shapes additive is why every field is public data, not behaviour.

use omm_ecs_core::EntityId;
use omm_protocol::{AccountId, CharacterId, ItemId, Tick};

/// A player just entered the world: the shard verified their session token,
/// loaded their durable character, and spawned its actor. Fired from the
/// authoritative accept handshake (`omm-shard`'s `accept_session`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerLoginCtx {
    /// The account the verified session token authorized.
    pub account: AccountId,
    /// The durable character that was loaded and spawned.
    pub character: CharacterId,
    /// The server-issued entity now driving that character in the world.
    pub entity: EntityId,
}

impl PlayerLoginCtx {
    /// Bundle a login event's participants.
    #[must_use]
    pub const fn new(account: AccountId, character: CharacterId, entity: EntityId) -> Self {
        Self {
            account,
            character,
            entity,
        }
    }
}

/// A creature (or player actor) died this tick. `killer` is `None` for a death
/// with no attributable source (environment, decay).
///
/// Defined so the hook surface is complete; the sim does not yet raise deaths as
/// events, so no core site fires this today (documented in
/// docs/architecture/10-modules.md). It compiles, dispatches, and is tested,
/// ready to wire the moment combat emits deaths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureDeathCtx {
    /// The entity that died.
    pub victim: EntityId,
    /// The entity credited with the kill, if any.
    pub killer: Option<EntityId>,
}

impl CreatureDeathCtx {
    /// Bundle a death event.
    #[must_use]
    pub const fn new(victim: EntityId, killer: Option<EntityId>) -> Self {
        Self { victim, killer }
    }
}

/// An item was looted: `looter` took `item` from `source`.
///
/// Like [`CreatureDeathCtx`], defined and tested ahead of a core loot system so
/// the API is stable; no core site fires it yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootCtx {
    /// The entity that received the loot.
    pub looter: EntityId,
    /// The entity or container the loot came from.
    pub source: EntityId,
    /// The item instance that changed hands.
    pub item: ItemId,
}

impl LootCtx {
    /// Bundle a loot event.
    #[must_use]
    pub const fn new(looter: EntityId, source: EntityId, item: ItemId) -> Self {
        Self {
            looter,
            source,
            item,
        }
    }
}

/// One fixed simulation step just completed. Fired every tick from the shard's
/// tick loop, after `World::step`, so a module sees the post-step world time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickCtx {
    /// The tick the world is now stamped at (after the step that fired this).
    pub tick: Tick,
    /// The fixed timestep in seconds — always the same value, deterministically.
    pub dt: f32,
}

impl TickCtx {
    /// Bundle a tick event.
    #[must_use]
    pub const fn new(tick: Tick, dt: f32) -> Self {
        Self { tick, dt }
    }
}
