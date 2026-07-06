//! `hello-world` — the proof-of-concept compiled module.
//!
//! It exists to prove the whole path end to end: a self-contained crate under
//! `modules/`, implementing the core [`ServerHooks`], is auto-discovered,
//! force-linked into the server, and *fires on real core events* — no core file
//! was edited to add it. It hooks the two events the shard raises today:
//! [`ServerHooks::on_player_login`] (from the authoritative accept handshake)
//! and [`ServerHooks::on_tick`] (from the fixed-timestep loop).
//!
//! Copy this as the template for a real module; `bin/new-module <name>`
//! generates the same shape from scratch.

use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};

use omm_module_api::{
    declare_module, Module, ModuleManifest, PlayerLoginCtx, ServerHooks, TickCtx,
};

/// The module's live state. Hooks take `&self`, so counters use atomics — which
/// also keeps the module `Send + Sync` for the shared tick thread.
#[derive(Debug, Default)]
pub struct HelloWorld {
    /// Player logins observed since boot.
    logins: AtomicU64,
    /// Ticks observed since boot.
    ticks: AtomicU64,
}

impl HelloWorld {
    /// Logins this module has seen — lets tooling and tests confirm the hook
    /// actually fired on a real event, not just that the code compiled in.
    #[must_use]
    pub fn logins(&self) -> u64 {
        self.logins.load(Ordering::Relaxed)
    }

    /// Ticks this module has seen since boot.
    #[must_use]
    pub fn ticks(&self) -> u64 {
        self.ticks.load(Ordering::Relaxed)
    }
}

impl ServerHooks for HelloWorld {
    fn on_player_login(&self, ctx: &PlayerLoginCtx) {
        let total = self.logins.fetch_add(1, Ordering::Relaxed) + 1;
        tracing::info!(
            target: "module::hello_world",
            account = ctx.account.raw(),
            character = ctx.character.raw(),
            entity = ctx.entity.0,
            total,
            "hello-world: a player entered the world",
        );
    }

    fn on_tick(&self, ctx: &TickCtx) {
        self.ticks.fetch_add(1, Ordering::Relaxed);
        // Ticks fire 30×/s — keep this at trace so the demo never floods logs.
        tracing::trace!(
            target: "module::hello_world",
            tick = ctx.tick.0,
            "hello-world: tick",
        );
    }
}

impl Module for HelloWorld {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest::new("hello-world", env!("CARGO_PKG_VERSION"))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Emit the `module()` entry point the generated `omm-modules` registry links to.
declare_module!(HelloWorld::default());

#[cfg(test)]
mod tests {
    use omm_ecs_core::EntityId;
    use omm_protocol::{AccountId, CharacterId, Tick};

    use super::*;

    fn login() -> PlayerLoginCtx {
        PlayerLoginCtx::new(AccountId::new(7), CharacterId::new(9), EntityId(11))
    }

    #[test]
    fn login_hook_counts_each_event() {
        let m = HelloWorld::default();
        assert_eq!(m.logins(), 0);
        m.on_player_login(&login());
        m.on_player_login(&login());
        assert_eq!(m.logins(), 2);
    }

    #[test]
    fn tick_hook_counts_each_event() {
        let m = HelloWorld::default();
        m.on_tick(&TickCtx::new(Tick(1), 1.0 / 30.0));
        assert_eq!(m.ticks(), 1);
    }

    #[test]
    fn module_entry_point_reports_manifest() {
        let boxed = module();
        assert_eq!(boxed.manifest().name, "hello-world");
        assert!(boxed.manifest().is_compatible());
    }
}
