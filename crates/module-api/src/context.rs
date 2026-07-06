//! Hook contexts — the immutable event payloads the core hands each hook.
//!
//! Each server event has one `*Ctx` carrying exactly what a module needs to
//! react, in typed ids (never a raw `u64` — the type system is the first
//! anti-cheat layer, and it does not stop at the module seam). Contexts are
//! read-only by design: this foundation exposes *observation* hooks. A later
//! slice can add a `&mut` world handle to contexts that must mutate; keeping the
//! shapes additive is why every field is public data, not behaviour.

use omm_ecs_core::EntityId;
use omm_protocol::{AccountId, CharacterId, ItemDefId, ItemId, Tick, ZoneId};

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

/// The channel a chat line was sent on — it decides who hears it and lets a
/// module react per scope (a `Say` ripples only to nearby actors; a `Guild`
/// line reaches the whole guild; a `Whisper` is one recipient).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatChannel {
    /// Local, proximity-limited speech.
    Say,
    /// Local speech at a wider radius.
    Yell,
    /// Everyone in the speaker's current zone.
    Zone,
    /// The speaker's party.
    Party,
    /// The speaker's guild.
    Guild,
    /// A single named recipient.
    Whisper,
    /// The cross-world trade channel.
    Trade,
}

/// A character level. A newtype over `u16` so a level is never confused with
/// another count and so ordered comparisons (`from < to`) are typed and clear.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Level(pub u16);

impl Level {
    /// Wrap a raw level value.
    #[must_use]
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    /// The underlying raw level.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// A chat message was sent. `message` is borrowed from the core's decode buffer
/// for the duration of the call, so a module reads it without allocating and the
/// lifetime forbids retaining it past the event — observation only, by design.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatCtx<'a> {
    /// The entity that sent the message.
    pub speaker: EntityId,
    /// The channel it was sent on.
    pub channel: ChatChannel,
    /// The message text, borrowed for the call.
    pub message: &'a str,
}

impl<'a> ChatCtx<'a> {
    /// Bundle a chat event.
    #[must_use]
    pub const fn new(speaker: EntityId, channel: ChatChannel, message: &'a str) -> Self {
        Self {
            speaker,
            channel,
            message,
        }
    }
}

/// A character's level changed. `from`/`to` bracket the transition (`to > from`
/// on a level-up), so a module can award exactly the newly-crossed levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelUpCtx {
    /// The entity whose level changed.
    pub entity: EntityId,
    /// The durable character behind it.
    pub character: CharacterId,
    /// The level held before this event.
    pub from: Level,
    /// The level held after it.
    pub to: Level,
}

impl LevelUpCtx {
    /// Bundle a level-up event.
    #[must_use]
    pub const fn new(entity: EntityId, character: CharacterId, from: Level, to: Level) -> Self {
        Self {
            entity,
            character,
            from,
            to,
        }
    }
}

/// A character crossed a zone boundary. `from` is `None` on the first zone
/// entered after login (there is no prior zone) and `Some(prev)` on a transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZoneEnterCtx {
    /// The entity that entered the zone.
    pub entity: EntityId,
    /// The durable character behind it.
    pub character: CharacterId,
    /// The zone left, or `None` on the first entry after login.
    pub from: Option<ZoneId>,
    /// The zone now entered.
    pub to: ZoneId,
}

impl ZoneEnterCtx {
    /// Bundle a zone-enter event.
    #[must_use]
    pub const fn new(
        entity: EntityId,
        character: CharacterId,
        from: Option<ZoneId>,
        to: ZoneId,
    ) -> Self {
        Self {
            entity,
            character,
            from,
            to,
        }
    }
}

/// An item was used (consumed, activated). `def` is the template, so a module
/// keys behaviour on the *kind* of item; `item` is the concrete instance used;
/// `target` is the actor it was used on, or `None` for a self/untargeted use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemUseCtx {
    /// The entity that used the item.
    pub user: EntityId,
    /// The concrete item instance consumed/activated.
    pub item: ItemId,
    /// The item template, for keying behaviour by kind.
    pub def: ItemDefId,
    /// The actor it was used on, or `None` if self/untargeted.
    pub target: Option<EntityId>,
}

impl ItemUseCtx {
    /// Bundle an item-use event.
    #[must_use]
    pub const fn new(
        user: EntityId,
        item: ItemId,
        def: ItemDefId,
        target: Option<EntityId>,
    ) -> Self {
        Self {
            user,
            item,
            def,
            target,
        }
    }
}
