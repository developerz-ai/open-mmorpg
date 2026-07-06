use super::*;
use bevy_ecs::prelude::{Component, ReflectComponent};
use bevy_reflect::{Reflect, TypePath, TypeRegistry, Typed};

// --- fixtures ---------------------------------------------------------------

#[derive(Reflect)]
struct Stats {
    hp: u32,
    speed: f32,
    name: String,
    alive: bool,
}

#[derive(Reflect)]
struct Wrapper(u32, String);

#[derive(Reflect)]
#[allow(dead_code)] // variants exist for reflection, never constructed
enum Mode {
    Idle,
    Walk(f32),
    Attack { power: u32, ranged: bool },
}

#[derive(Reflect)]
struct WithVec {
    items: Vec<u32>,
}

#[derive(Reflect)]
struct WithMap {
    table: std::collections::HashMap<String, u32>,
}

#[derive(Reflect, Clone)]
#[reflect(opaque)]
struct Blob(#[allow(dead_code)] u64);

#[derive(Reflect)]
struct Recur {
    next: Vec<Recur>,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct CompA {
    x: u32,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct CompZ {
    y: bool,
}

#[derive(Reflect, Default)]
#[allow(dead_code)]
struct PlainData {
    z: f32,
}

// --- helpers ----------------------------------------------------------------

fn child<'a>(d: &'a WidgetDescriptor, label: &str) -> &'a WidgetDescriptor {
    d.children
        .iter()
        .find(|c| c.label == label)
        .unwrap_or_else(|| panic!("no child labelled `{label}` in {d:?}"))
}

fn has_truncated(d: &WidgetDescriptor) -> bool {
    d.truncated || d.children.iter().any(has_truncated)
}

// --- primitives & structs ---------------------------------------------------

#[test]
fn primitive_fields_map_to_leaf_widgets() {
    let d = describe_type(Stats::type_info());
    assert_eq!(d.kind, WidgetKind::Struct);
    assert_eq!(d.label, Stats::type_path());
    assert_eq!(d.children.len(), 4);
    assert_eq!(child(&d, "hp").kind, WidgetKind::Integer);
    assert_eq!(child(&d, "speed").kind, WidgetKind::Float);
    assert_eq!(child(&d, "name").kind, WidgetKind::Text);
    assert_eq!(child(&d, "alive").kind, WidgetKind::Checkbox);
    // Leaves carry no children.
    assert!(child(&d, "hp").children.is_empty());
}

#[test]
fn tuple_struct_fields_are_indexed() {
    let d = describe_type(Wrapper::type_info());
    assert_eq!(d.kind, WidgetKind::Struct);
    assert_eq!(d.children.len(), 2);
    assert_eq!(child(&d, "0").kind, WidgetKind::Integer);
    assert_eq!(child(&d, "1").kind, WidgetKind::Text);
}

// --- enums ------------------------------------------------------------------

#[test]
fn enum_variants_become_groups() {
    let d = describe_type(Mode::type_info());
    assert_eq!(d.kind, WidgetKind::Enum);
    assert_eq!(d.children.len(), 3);

    let idle = child(&d, "Idle");
    assert_eq!(idle.kind, WidgetKind::Struct);
    assert!(idle.children.is_empty());
    assert!(idle.type_path.ends_with("::Idle"));

    let walk = child(&d, "Walk");
    assert_eq!(walk.children.len(), 1);
    assert_eq!(child(walk, "0").kind, WidgetKind::Float);

    let attack = child(&d, "Attack");
    assert_eq!(attack.children.len(), 2);
    assert_eq!(child(attack, "power").kind, WidgetKind::Integer);
    assert_eq!(child(attack, "ranged").kind, WidgetKind::Checkbox);
}

// --- collections ------------------------------------------------------------

#[test]
fn vec_field_is_a_list_template() {
    let d = describe_type(WithVec::type_info());
    let items = child(&d, "items");
    assert_eq!(items.kind, WidgetKind::List);
    assert_eq!(items.children.len(), 1);
    let template = child(items, "item");
    assert_eq!(template.kind, WidgetKind::Integer);
}

#[test]
fn map_field_has_key_and_value_templates() {
    let d = describe_type(WithMap::type_info());
    let table = child(&d, "table");
    assert_eq!(table.kind, WidgetKind::Map);
    assert_eq!(child(table, "key").kind, WidgetKind::Text);
    assert_eq!(child(table, "value").kind, WidgetKind::Integer);
}

#[test]
fn opaque_type_is_a_read_only_leaf() {
    let d = describe_type(Blob::type_info());
    assert_eq!(d.kind, WidgetKind::Opaque);
    assert!(d.children.is_empty());
    assert!(!d.truncated);
}

// --- registry-driven surface ------------------------------------------------

#[test]
fn inspect_components_lists_only_components_sorted() {
    let mut registry = TypeRegistry::new();
    // Register out of order + a non-component to prove filtering and sorting.
    registry.register::<CompZ>();
    registry.register::<PlainData>();
    registry.register::<CompA>();

    let out = inspect_components(&registry);
    assert_eq!(
        out.len(),
        2,
        "only #[reflect(Component)] types are inspectable"
    );
    assert!(out[0].type_path.ends_with("::CompA"));
    assert!(out[1].type_path.ends_with("::CompZ"));
    assert!(out.iter().all(|d| !d.type_path.ends_with("::PlainData")));
    // Field widgets are generated for the component too.
    assert_eq!(child(&out[0], "x").kind, WidgetKind::Integer);
}

#[test]
fn describe_by_path_resolves_registered_and_rejects_unknown() {
    let mut registry = TypeRegistry::new();
    registry.register::<CompA>();
    let d = describe_by_path(&registry, CompA::type_path()).expect("registered type resolves");
    assert_eq!(d.type_path, CompA::type_path());
    assert!(describe_by_path(&registry, "does::not::Exist").is_none());
}

// --- invariants -------------------------------------------------------------

#[test]
fn descriptor_is_deterministic() {
    // Same type → identical descriptor, and enumeration is stably ordered.
    assert_eq!(
        describe_type(Stats::type_info()),
        describe_type(Stats::type_info())
    );

    let mut registry = TypeRegistry::new();
    registry.register::<CompZ>();
    registry.register::<CompA>();
    assert_eq!(inspect_components(&registry), inspect_components(&registry));
}

#[test]
fn descriptor_round_trips_through_json() {
    // The MCP boundary is JSON — the descriptor must survive it losslessly.
    let d = describe_type(Stats::type_info());
    let json = serde_json::to_string(&d).expect("serialize");
    let back: WidgetDescriptor = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(d, back);
    // snake_case kind tags, loud and stable for agents.
    assert!(json.contains("\"kind\":\"struct\""));
    assert!(json.contains("\"checkbox\""));
}

#[test]
fn recursive_type_truncates_instead_of_overflowing() {
    // Reaching this assertion at all proves recursion terminated.
    let d = describe_type(Recur::type_info());
    assert!(
        has_truncated(&d),
        "self-referential type must truncate at MAX_DEPTH"
    );
}
