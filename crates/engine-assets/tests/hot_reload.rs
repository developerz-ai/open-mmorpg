//! Integration tests for the hot-reload channel and the `AssetsPlugin` registry.
//!
//! Verifies that:
//!  - the plugin registers the `AssetHotReloaded` message in the world
//!  - a simulated in-place asset mutation emits exactly one `AssetHotReloaded`
//!  - `Added` and `Removed` events are _not_ forwarded as hot reloads
//!  - a second `forward_hot_reload` registration for the same type does not crash

use bevy_app::App;
use bevy_asset::{Asset, AssetApp, Assets};
use bevy_ecs::{
    message::Messages,
    prelude::{ResMut, Resource},
};
use bevy_reflect::TypePath;
use omm_engine_assets::{AssetHotReloadAppExt, AssetHotReloaded, AssetsPlugin};
use omm_engine_core::EnginePlugins;

// ── throwaway asset ───────────────────────────────────────────────────────────

/// Minimal asset type that only exists to exercise the hot-reload forwarder
/// without any render types or GPU involvement.
#[derive(Asset, TypePath, Default)]
struct FakeAsset {
    version: u32,
}

// ── capture helper ────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct Captured(Vec<AssetHotReloaded>);

fn capture_reloads(
    mut sink: ResMut<Captured>,
    mut reader: bevy_ecs::message::MessageReader<AssetHotReloaded>,
) {
    for evt in reader.read() {
        sink.0.push(evt.clone());
    }
}

fn build_app() -> App {
    let mut app = App::new();
    app.add_plugins(EnginePlugins);
    app.add_plugins(AssetsPlugin::default());
    app.init_asset::<FakeAsset>();
    app.forward_hot_reload::<FakeAsset>();
    app.init_resource::<Captured>();
    // `Last` runs after the `PostUpdate` forwarder — captures same-frame reloads.
    app.add_systems(bevy_app::Last, capture_reloads);
    app
}

// ── registry ──────────────────────────────────────────────────────────────────

/// The plugin must register `Messages<AssetHotReloaded>` in the ECS world so
/// that listeners can be wired before the first frame without panicking.
#[test]
fn hot_reload_message_is_registered() {
    let app = build_app();
    assert!(
        app.world()
            .get_resource::<Messages<AssetHotReloaded>>()
            .is_some(),
        "AssetHotReloaded message type must be registered by AssetsPlugin"
    );
}

// ── simulated change emits the event ─────────────────────────────────────────

/// Mutating an asset in-place (simulating what a file-watcher reload does)
/// must emit exactly one `AssetHotReloaded` with the correct type name.
#[test]
fn modifying_asset_emits_hot_reload_event() {
    let mut app = build_app();

    let handle = app
        .world_mut()
        .resource_mut::<Assets<FakeAsset>>()
        .add(FakeAsset::default());

    // Frame 1: flushes `Added` — must NOT be forwarded.
    app.update();
    assert!(
        app.world().resource::<Captured>().0.is_empty(),
        "Added must not surface as a hot-reload"
    );

    // Mutate in-place → Bevy emits `AssetEvent::Modified`.
    {
        let mut assets = app.world_mut().resource_mut::<Assets<FakeAsset>>();
        let mut asset = assets.get_mut(&handle).expect("asset present");
        asset.version += 1;
    }

    // Frame 2: the forwarder picks up the Modified event.
    app.update();

    let captured = &app.world().resource::<Captured>().0;
    assert_eq!(
        captured.len(),
        1,
        "exactly one modify → one hot-reload signal"
    );
    let reload = captured.first().expect("captured");
    assert_eq!(
        reload.asset_type,
        core::any::type_name::<FakeAsset>(),
        "asset_type must name the concrete type"
    );
}

/// A second in-place mutation in the same run must fire another event.
#[test]
fn each_modification_emits_a_separate_event() {
    let mut app = build_app();
    let handle = app
        .world_mut()
        .resource_mut::<Assets<FakeAsset>>()
        .add(FakeAsset::default());
    app.update(); // flush Added

    for _ in 0..2 {
        {
            let mut assets = app.world_mut().resource_mut::<Assets<FakeAsset>>();
            let mut asset = assets.get_mut(&handle).expect("asset present");
            asset.version += 1;
        }
        app.update();
    }

    assert_eq!(
        app.world().resource::<Captured>().0.len(),
        2,
        "two modifications must yield two hot-reload signals"
    );
}

// ── Added / Removed are not hot reloads ──────────────────────────────────────

#[test]
fn added_and_removed_are_not_hot_reloads() {
    let mut app = build_app();
    let handle = app
        .world_mut()
        .resource_mut::<Assets<FakeAsset>>()
        .add(FakeAsset::default());
    app.update(); // Added

    app.world_mut()
        .resource_mut::<Assets<FakeAsset>>()
        .remove(&handle);
    app.update(); // Removed

    assert!(
        app.world().resource::<Captured>().0.is_empty(),
        "Add and Remove lifecycle events must not produce hot-reload signals"
    );
}

// ── duplicate registration is harmless ───────────────────────────────────────

/// Registering `forward_hot_reload` for the same type twice must not panic.
/// (Bevy systems are idempotent at scheduling level; a double-add just runs
/// the system once more per frame, which is an acceptable no-op for this test.)
#[test]
fn duplicate_forward_registration_does_not_panic() {
    let mut app = App::new();
    app.add_plugins(EnginePlugins);
    app.add_plugins(AssetsPlugin::default());
    app.init_asset::<FakeAsset>();
    app.forward_hot_reload::<FakeAsset>();
    app.forward_hot_reload::<FakeAsset>(); // second registration
    app.finish();
    app.cleanup();
    app.update(); // must not panic
}
