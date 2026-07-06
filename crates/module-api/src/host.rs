//! [`ModuleHost`] — the registry the core holds and calls.
//!
//! The host owns every loaded [`Module`] and is itself a [`ServerHooks`]: the
//! core calls one hook on the host, and the host fans it out to every module in
//! **registration order**. That order is deterministic — the generated registry
//! registers modules in sorted directory order — so two shards replaying the
//! same events run module hooks in the same sequence, preserving the
//! determinism the anti-cheat re-sim depends on.
//!
//! An empty host (no modules) is a valid, total no-op: the core wires the host
//! unconditionally, and a build with zero modules simply does nothing on every
//! hook.

use crate::context::{
    ChatCtx, CreatureDeathCtx, ItemUseCtx, LevelUpCtx, LootCtx, PlayerLoginCtx, TickCtx,
    ZoneEnterCtx,
};
use crate::hooks::ServerHooks;
use crate::manifest::ModuleManifest;
use crate::module::Module;

/// The set of loaded modules the core dispatches to.
#[derive(Default)]
pub struct ModuleHost {
    /// Modules in registration (deterministic) order.
    modules: Vec<Box<dyn Module>>,
}

impl ModuleHost {
    /// An empty host — a valid no-op until modules are registered.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a module. Called by the generated registry once per linked-in
    /// module, in sorted-directory order; returns `&mut self` for chaining.
    pub fn register(&mut self, module: Box<dyn Module>) -> &mut Self {
        self.modules.push(module);
        self
    }

    /// The number of loaded modules.
    #[must_use]
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Whether no module is loaded (every hook is then a no-op).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// The identity of every loaded module, in dispatch order — for a boot log
    /// or an operator's module list.
    pub fn manifests(&self) -> impl Iterator<Item = ModuleManifest> + '_ {
        self.modules.iter().map(|m| m.manifest())
    }

    /// The first loaded module of concrete type `T`, if any.
    ///
    /// Downcasts through [`Module::as_any`]. Lets tooling recover a module to
    /// inspect its state, and lets an integration test assert a real event
    /// reached the real, linked-in module — not a stand-in.
    #[must_use]
    pub fn get<T: Module>(&self) -> Option<&T> {
        self.modules
            .iter()
            .find_map(|m| m.as_any().downcast_ref::<T>())
    }
}

/// Fan every hook out to each module in order. The host *is* the core's single
/// hook surface, so adding a module never changes a core call site.
impl ServerHooks for ModuleHost {
    fn on_player_login(&self, ctx: &PlayerLoginCtx) {
        for module in &self.modules {
            module.on_player_login(ctx);
        }
    }

    fn on_creature_death(&self, ctx: &CreatureDeathCtx) {
        for module in &self.modules {
            module.on_creature_death(ctx);
        }
    }

    fn on_loot(&self, ctx: &LootCtx) {
        for module in &self.modules {
            module.on_loot(ctx);
        }
    }

    fn on_tick(&self, ctx: &TickCtx) {
        for module in &self.modules {
            module.on_tick(ctx);
        }
    }

    fn on_chat(&self, ctx: &ChatCtx<'_>) {
        for module in &self.modules {
            module.on_chat(ctx);
        }
    }

    fn on_level_up(&self, ctx: &LevelUpCtx) {
        for module in &self.modules {
            module.on_level_up(ctx);
        }
    }

    fn on_zone_enter(&self, ctx: &ZoneEnterCtx) {
        for module in &self.modules {
            module.on_zone_enter(ctx);
        }
    }

    fn on_item_use(&self, ctx: &ItemUseCtx) {
        for module in &self.modules {
            module.on_item_use(ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::sync::atomic::{AtomicU64, Ordering};

    use omm_ecs_core::EntityId;
    use omm_protocol::{AccountId, CharacterId, Tick};

    use super::*;
    use crate::context::Level;

    /// A fake module counting the hooks that fired on it.
    #[derive(Default)]
    struct Spy {
        name: &'static str,
        logins: AtomicU64,
        ticks: AtomicU64,
        level_ups: AtomicU64,
    }

    impl ServerHooks for Spy {
        fn on_player_login(&self, _ctx: &PlayerLoginCtx) {
            self.logins.fetch_add(1, Ordering::Relaxed);
        }
        fn on_tick(&self, _ctx: &TickCtx) {
            self.ticks.fetch_add(1, Ordering::Relaxed);
        }
        fn on_level_up(&self, _ctx: &LevelUpCtx) {
            self.level_ups.fetch_add(1, Ordering::Relaxed);
        }
    }

    impl Module for Spy {
        fn manifest(&self) -> ModuleManifest {
            ModuleManifest::new(self.name, "0.0.0")
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    fn login() -> PlayerLoginCtx {
        PlayerLoginCtx::new(AccountId::new(1), CharacterId::new(2), EntityId(3))
    }

    fn level_up() -> LevelUpCtx {
        LevelUpCtx::new(
            EntityId(3),
            CharacterId::new(2),
            Level::new(4),
            Level::new(5),
        )
    }

    #[test]
    fn empty_host_is_a_total_noop() {
        let host = ModuleHost::new();
        assert!(host.is_empty());
        assert_eq!(host.len(), 0);
        // Every hook is safe to call with no modules.
        host.on_player_login(&login());
        host.on_tick(&TickCtx::new(Tick(0), 0.0));
        assert_eq!(host.manifests().count(), 0);
    }

    #[test]
    fn register_dispatches_to_every_module() {
        let mut host = ModuleHost::new();
        host.register(Box::new(Spy {
            name: "a",
            ..Spy::default()
        }))
        .register(Box::new(Spy {
            name: "b",
            ..Spy::default()
        }));
        assert_eq!(host.len(), 2);

        host.on_player_login(&login());
        // Both modules saw the one event.
        for m in host.get_all() {
            assert_eq!(m.logins.load(Ordering::Relaxed), 1);
        }
    }

    #[test]
    fn a_new_hook_dispatches_to_every_module() {
        let mut host = ModuleHost::new();
        host.register(Box::new(Spy {
            name: "a",
            ..Spy::default()
        }))
        .register(Box::new(Spy {
            name: "b",
            ..Spy::default()
        }));

        host.on_level_up(&level_up());

        // A hook added after the foundation fans out through the host exactly
        // like the original ones — every registered module observed the event.
        for m in host.get_all() {
            assert_eq!(m.level_ups.load(Ordering::Relaxed), 1);
        }
    }

    #[test]
    fn get_downcasts_to_a_concrete_module() {
        let mut host = ModuleHost::new();
        host.register(Box::new(Spy {
            name: "only",
            ..Spy::default()
        }));
        let spy = host.get::<Spy>().expect("the Spy is registered");
        assert_eq!(spy.manifest().name, "only");
    }

    #[test]
    fn manifests_list_in_registration_order() {
        let mut host = ModuleHost::new();
        host.register(Box::new(Spy {
            name: "first",
            ..Spy::default()
        }))
        .register(Box::new(Spy {
            name: "second",
            ..Spy::default()
        }));
        let names: Vec<_> = host.manifests().map(|m| m.name).collect();
        assert_eq!(names, ["first", "second"]);
    }

    // Test-only helper: iterate the concrete Spies to check per-module state.
    impl ModuleHost {
        fn get_all(&self) -> impl Iterator<Item = &Spy> {
            self.modules
                .iter()
                .filter_map(|m| m.as_any().downcast_ref())
        }
    }
}
