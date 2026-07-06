//! [`ServerHooks`] — the trait the core calls at each server event.
//!
//! Every method has a default no-op body, so a module `impl`s only the events it
//! cares about and the core can call any hook unconditionally. Hooks take
//! `&self`: a module is a long-lived observer shared across the tick thread, so
//! it holds any state behind interior mutability (an `AtomicU64`, a `Mutex`) —
//! which also keeps every hook `Send + Sync` and the whole registry cheap to
//! share. Bodies must stay non-blocking; they run on the authoritative tick
//! path, where a stall or panic would drop every player on the shard.

use crate::context::{
    ChatCtx, CreatureDeathCtx, ItemUseCtx, LevelUpCtx, LootCtx, PlayerLoginCtx, TickCtx,
    ZoneEnterCtx,
};

/// The server-side hook surface. The core owns the events; a module reacts.
///
/// `Send + Sync` because the [`crate::ModuleHost`] holding these is shared with
/// the tick loop. Default methods are no-ops so overriding one hook never forces
/// stubbing the rest, and so adding a new hook here is backward-compatible for
/// every existing module.
///
/// The surface spans a player's life on the shard: lifecycle
/// ([`ServerHooks::on_player_login`]), social ([`ServerHooks::on_chat`]),
/// progression ([`ServerHooks::on_level_up`], [`ServerHooks::on_zone_enter`]),
/// economy ([`ServerHooks::on_loot`], [`ServerHooks::on_item_use`]), combat
/// ([`ServerHooks::on_creature_death`]), and the per-tick step
/// ([`ServerHooks::on_tick`]).
pub trait ServerHooks: Send + Sync {
    /// A player finished the accept handshake and their actor is live.
    fn on_player_login(&self, _ctx: &PlayerLoginCtx) {}

    /// An actor died. `killer` is `None` when no source is attributable.
    fn on_creature_death(&self, _ctx: &CreatureDeathCtx) {}

    /// An item was looted from a source into a looter.
    fn on_loot(&self, _ctx: &LootCtx) {}

    /// One fixed simulation tick completed.
    fn on_tick(&self, _ctx: &TickCtx) {}

    /// A chat message was sent on some channel.
    fn on_chat(&self, _ctx: &ChatCtx<'_>) {}

    /// A character's level changed (`ctx.to` above `ctx.from` on a level-up).
    fn on_level_up(&self, _ctx: &LevelUpCtx) {}

    /// A character entered a zone (`ctx.from` is `None` on the first entry).
    fn on_zone_enter(&self, _ctx: &ZoneEnterCtx) {}

    /// An item instance was used, optionally on a target.
    fn on_item_use(&self, _ctx: &ItemUseCtx) {}
}
