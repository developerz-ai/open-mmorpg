//! Reusable Bevy-based **engine core** — the headless-first foundation both the
//! rendered client and the authoritative server stand on.
//!
//! # One core, two heads
//! The engine is a *library of plugins*, not a framework you live inside. This
//! crate owns the app lifecycle and the ECS: [`EngineCorePlugin`] registers the
//! reflection type registry and the fixed-timestep simulation schedule, and
//! [`EnginePlugins`] bundles the minimal headless substrate (task pool + time +
//! core). Rendering, audio, physics and UI are plugins added *on top* — never a
//! dependency the core can't remove. Swap the render/window plugins out and the
//! exact same `App`/ECS/schedule runs headless in CI and in an agent harness.
//! → `docs/specs/game-engine/core/README.md`.
//!
//! # Determinism where shared
//! Simulation lives in [`FixedUpdate`] (fixed `dt`), decoupled from any render
//! timestep, so headless results equal headful results. [`headless_app`] pins a
//! manual time step equal to the fixed timestep, making each `App::update()`
//! advance the sim by exactly one deterministic tick — the property the server
//! re-simulation and replay depend on.

pub mod run;

pub use run::{headless_run, run_ticks};

use bevy_app::prelude::*;
use bevy_app::PluginGroupBuilder;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use bevy_time::{Fixed, Time, TimePlugin, TimeUpdateStrategy};

/// Fixed simulation rate, in Hertz. Matches the shard/client tick rate so the
/// engine's `FixedUpdate` steps align with the authoritative sim. Fixed, never
/// wall-clock — that is what keeps the simulation deterministic.
pub const TICK_HZ: f64 = 30.0;

/// Monotonic count of fixed simulation steps the app has run.
///
/// A tiny engine-level heartbeat: incremented once per [`FixedUpdate`], it
/// proves the fixed schedule advances and gives headless runners something
/// deterministic to assert against. Registered in the reflection registry, so
/// tools and agents can enumerate and read it like any other type.
#[derive(Resource, Reflect, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[reflect(Resource)]
pub struct FixedTick(pub u64);

/// Advance the fixed-tick heartbeat by one. Runs in [`FixedUpdate`], so it fires
/// exactly once per fixed simulation step. Saturating: never panics on overflow.
fn advance_fixed_tick(mut tick: ResMut<FixedTick>) {
    tick.0 = tick.0.saturating_add(1);
}

/// The engine's core plugin: reflection registry + fixed-timestep simulation.
///
/// Headless-safe — pulls in no rendering, windowing, audio or GPU. Add it (via
/// [`EnginePlugins`]) to any `App` to get a deterministic fixed schedule and a
/// populated type registry.
#[derive(Debug, Default, Clone, Copy)]
pub struct EngineCorePlugin;

impl Plugin for EngineCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(TICK_HZ))
            .init_resource::<FixedTick>()
            .register_type::<FixedTick>()
            .add_systems(FixedUpdate, advance_fixed_tick);
    }
}

/// The minimal **headless** engine plugin group: task pool (parallel schedule),
/// time (drives `FixedUpdate`), and [`EngineCorePlugin`].
///
/// This is the reusable substrate a *different* game can depend on with no core
/// fork. The rendered client composes render/window/audio/UI plugins on top of
/// this same group; the server uses it as-is for headless simulation.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnginePlugins;

impl PluginGroup for EnginePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(TaskPoolPlugin::default())
            .add(TimePlugin)
            .add(EngineCorePlugin)
    }
}

/// Build a fully headless, deterministic [`App`] — no GPU, no window, no audio.
///
/// Adds [`EnginePlugins`] and pins the time-update strategy to a manual step
/// equal to the fixed timestep, so every `App::update()` advances the simulation
/// by exactly one fixed tick. This is the app CI, tests and agent harnesses
/// drive; see [`run_ticks`] for the run-N-ticks-and-exit runner.
pub fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins(EnginePlugins);
    // Read back the exact fixed timestep the core installed so the manual step
    // matches it bit-for-bit: each update() then runs exactly one FixedUpdate.
    let dt = app.world().resource::<Time<Fixed>>().timestep();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(dt));
    app
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::AppTypeRegistry;
    use core::any::TypeId;

    #[test]
    fn headless_app_boots() {
        // No panic building the app means the headless substrate composes.
        let app = headless_app();
        assert_eq!(app.world().resource::<FixedTick>().0, 0);
    }

    #[test]
    fn fixed_timestep_is_pinned_to_tick_hz() {
        let app = headless_app();
        let dt = app.world().resource::<Time<Fixed>>().timestep();
        // 30 Hz => ~33.3ms per fixed step.
        assert!((dt.as_secs_f64() - 1.0 / TICK_HZ).abs() < 1e-9);
    }

    #[test]
    fn manual_time_step_equals_fixed_timestep() {
        // Determinism hinges on this: one update() == one fixed tick.
        let app = headless_app();
        let fixed = app.world().resource::<Time<Fixed>>().timestep();
        match app.world().resource::<TimeUpdateStrategy>() {
            TimeUpdateStrategy::ManualDuration(d) => assert_eq!(*d, fixed),
            _ => panic!("headless_app must pin a manual time step"),
        }
    }

    #[test]
    fn gameplay_types_are_registered_for_reflection() {
        // A type not in the registry is invisible to the editor, scene I/O and
        // agents — registration is mandatory, not optional.
        let app = headless_app();
        let registry = app.world().resource::<AppTypeRegistry>().read();
        assert!(
            registry.get(TypeId::of::<FixedTick>()).is_some(),
            "FixedTick must be registered in the reflection type registry"
        );
    }
}
