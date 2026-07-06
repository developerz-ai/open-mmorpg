//! [`ServerHooks`] — the trait the core calls at each server event.
//!
//! Every method has a default no-op body, so a module `impl`s only the events it
//! cares about and the core can call any hook unconditionally. Hooks take
//! `&self`: a module is a long-lived observer shared across the tick thread, so
//! it holds any state behind interior mutability (an `AtomicU64`, a `Mutex`) —
//! which also keeps every hook `Send + Sync` and the whole registry cheap to
//! share. Bodies must stay non-blocking; they run on the authoritative tick
//! path, where a stall or panic would drop every player on the shard.

use crate::context::{CreatureDeathCtx, LootCtx, PlayerLoginCtx, TickCtx};

/// The server-side hook surface. The core owns the events; a module reacts.
///
/// `Send + Sync` because the [`crate::ModuleHost`] holding these is shared with
/// the tick loop. Default methods are no-ops so overriding one hook never forces
/// stubbing the rest, and so adding a new hook here is backward-compatible for
/// every existing module.
pub trait ServerHooks: Send + Sync {
    /// A player finished the accept handshake and their actor is live.
    fn on_player_login(&self, _ctx: &PlayerLoginCtx) {}

    /// An actor died. `killer` is `None` when no source is attributable.
    fn on_creature_death(&self, _ctx: &CreatureDeathCtx) {}

    /// An item was looted from a source into a looter.
    fn on_loot(&self, _ctx: &LootCtx) {}

    /// One fixed simulation tick completed.
    fn on_tick(&self, _ctx: &TickCtx) {}
}
