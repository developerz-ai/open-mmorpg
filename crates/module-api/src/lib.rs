//! `omm-module-api` — the compiled-module hook API.
//!
//! Forks extend the server by dropping a self-contained crate under `modules/`
//! that implements the hook traits here; the build links it in and the core
//! calls it. **No core file is edited to add a feature** — the whole point,
//! modelled on AzerothCore's module system but done in pure Cargo (compile-time
//! crates, not fragile runtime `.so` plugins, because the Rust ABI is unstable).
//!
//! The pieces:
//! - [`ServerHooks`] — the trait the core calls. One default-noop method per
//!   server event ([`ServerHooks::on_player_login`], [`ServerHooks::on_chat`],
//!   [`ServerHooks::on_level_up`], [`ServerHooks::on_zone_enter`],
//!   [`ServerHooks::on_item_use`], [`ServerHooks::on_tick`], …). A module
//!   overrides only the events it cares about.
//! - The `*Ctx` [`context`] payloads — the immutable event data each hook
//!   receives, carrying typed ids (never raw `u64`).
//! - [`Module`] — what a module *is*: its [`ModuleManifest`] identity plus the
//!   hooks it implements, behind an object-safe, downcastable trait object.
//! - [`ModuleHost`] — the registry the core holds. It fans every hook out to
//!   every registered module in a deterministic order and is itself a
//!   [`ServerHooks`], so the core calls one thing.
//! - [`declare_module!`] — one macro that emits the `module()` entry point the
//!   generated registry links against.
//!
//! Server-first by design; the same trait-plus-registry pattern extends to the
//! engine and client next (docs/architecture/10-modules.md).

pub mod context;
pub mod hooks;
pub mod host;
mod macros;
pub mod manifest;
pub mod module;

pub use context::{
    ChatChannel, ChatCtx, CreatureDeathCtx, ItemUseCtx, Level, LevelUpCtx, LootCtx, PlayerLoginCtx,
    TickCtx, ZoneEnterCtx,
};
pub use hooks::ServerHooks;
pub use host::ModuleHost;
pub use manifest::{ModuleManifest, CORE_API_VERSION};
pub use module::Module;
