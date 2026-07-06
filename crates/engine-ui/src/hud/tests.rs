use super::*;
use crate::inspector::{inspect_components, WidgetKind};
use bevy_app::App;
use bevy_ecs::prelude::AppTypeRegistry;
use bevy_ecs::schedule::Schedule;
use bevy_ecs::world::World;

fn en_bundle() -> I18nBundle {
    I18nBundle::from_sources("en", [("en", "rank-knight = Knight")]).expect("valid ftl")
}

// --- HealthBar math ---------------------------------------------------------

#[test]
fn fraction_clamps_and_is_robust() {
    assert_eq!(HealthBar::new(50.0, 100.0).fraction(), 0.5);
    assert_eq!(
        HealthBar::new(150.0, 100.0).fraction(),
        1.0,
        "over-heal clamps full"
    );
    assert_eq!(
        HealthBar::new(-5.0, 100.0).fraction(),
        0.0,
        "negative clamps empty"
    );
    assert_eq!(
        HealthBar::new(1.0, 0.0).fraction(),
        0.0,
        "zero max → empty, no div-by-zero"
    );
    assert_eq!(
        HealthBar::new(1.0, -1.0).fraction(),
        0.0,
        "negative max → empty"
    );
    assert_eq!(
        HealthBar::new(f32::NAN, 100.0).fraction(),
        0.0,
        "NaN current → empty"
    );
    assert_eq!(HealthBar::new(50.0, 100.0).fill(), Val::Percent(50.0));
}

#[test]
fn default_health_bar_is_full() {
    assert_eq!(HealthBar::default().fraction(), 1.0);
}

// --- retained health-bar widget ---------------------------------------------

#[test]
fn spawn_health_bar_sets_initial_width_and_components() {
    let mut world = World::new();
    let e = spawn_health_bar(
        &mut world.commands(),
        HealthBar::new(50.0, 100.0),
        Color::WHITE,
    );
    world.flush();
    assert_eq!(
        world.get::<Node>(e).expect("node").width,
        Val::Percent(50.0)
    );
    assert!(world.get::<HealthBar>(e).is_some());
    assert!(world.get::<BackgroundColor>(e).is_some());
}

#[test]
fn sync_health_bars_binds_width_to_data() {
    let mut world = World::new();
    // Node width starts wrong; the system must correct it from the data.
    let e = world
        .spawn((HealthBar::new(25.0, 100.0), Node::default()))
        .id();
    let mut schedule = Schedule::default();
    schedule.add_systems(sync_health_bars);
    schedule.run(&mut world);
    assert_eq!(
        world.get::<Node>(e).expect("node").width,
        Val::Percent(25.0)
    );

    // Mutating the data re-triggers the Changed filter.
    world.get_mut::<HealthBar>(e).expect("bar").current = 100.0;
    schedule.run(&mut world);
    assert_eq!(
        world.get::<Node>(e).expect("node").width,
        Val::Percent(100.0)
    );
}

// --- nameplate resolution ---------------------------------------------------

#[test]
fn nameplate_resolve_shows_name_and_translated_subtitle() {
    let i18n = en_bundle();
    assert_eq!(Nameplate::new("Ada").resolve(&i18n), "Ada");
    assert_eq!(
        Nameplate::new("Ada")
            .with_subtitle("rank-knight")
            .resolve(&i18n),
        "Ada\nKnight"
    );
    // Missing key renders loud, never blank.
    assert_eq!(
        Nameplate::new("Ada").with_subtitle("nope").resolve(&i18n),
        "Ada\n⟦nope⟧"
    );
}

#[test]
fn sync_nameplates_writes_resolved_text() {
    let mut world = World::new();
    world.insert_resource(en_bundle());
    let e = world
        .spawn((
            Nameplate::new("Ada").with_subtitle("rank-knight"),
            Text(String::new()),
        ))
        .id();
    let mut schedule = Schedule::default();
    schedule.add_systems(sync_nameplates);
    schedule.run(&mut world);
    assert_eq!(world.get::<Text>(e).expect("text").0, "Ada\nKnight");
}

#[test]
fn spawn_nameplate_starts_with_verbatim_name() {
    let mut world = World::new();
    let e = spawn_nameplate(&mut world.commands(), Nameplate::new("Ada"));
    world.flush();
    assert_eq!(world.get::<Text>(e).expect("text").0, "Ada");
    assert!(world.get::<Nameplate>(e).is_some());
}

// --- reflection ↔ HUD tie-in ------------------------------------------------

#[test]
fn hud_plugin_registers_widgets_for_reflection() {
    // The retained widgets must be inspectable by the same reflection surface
    // that feeds the editor/MCP — enumerating components should find them.
    let mut app = App::new();
    app.add_plugins(HudPlugin);
    let registry = app.world().resource::<AppTypeRegistry>().read();
    let descriptors = inspect_components(&registry);

    let health = descriptors
        .iter()
        .find(|d| d.type_path.ends_with("::HealthBar"))
        .expect("HealthBar registered + inspectable");
    assert_eq!(health.kind, WidgetKind::Struct);
    let current = health
        .children
        .iter()
        .find(|c| c.label == "current")
        .expect("current field");
    assert_eq!(current.kind, WidgetKind::Float);

    assert!(descriptors
        .iter()
        .any(|d| d.type_path.ends_with("::Nameplate")));
}
