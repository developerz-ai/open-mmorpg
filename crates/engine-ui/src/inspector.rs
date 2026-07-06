//! Reflection → **widget descriptors**: enumerate a reflected type's fields and
//! emit a UI-agnostic tree describing which editor widget renders each field.
//!
//! This is the E9→E10 bridge. The [editor](../../../docs/specs/game-engine/editor/README.md)
//! Details panel and the [MCP](../../../docs/specs/game-engine/ai-native-dx/README.md)
//! authoring surface are **generated from `bevy_reflect`**, not hand-built per
//! type: add a field to a registered component and it becomes editable
//! everywhere, with zero per-type UI code (the `UPROPERTY`→auto-UI pattern).
//!
//! It is **pure and headless** — it reads `TypeInfo` from the reflection
//! registry and returns owned, `serde`-serializable data. No GPU, no window, no
//! rendering. A [`WidgetDescriptor`] is the wire format an editor (GUI *or*
//! agent) consumes; whether a real widget is ever drawn is the renderer's job.
//!
//! # Contract
//! - Primitives (`bool`, integers, floats, `String`/`char`) map to leaf widgets
//!   ([`WidgetKind::Checkbox`] / [`Integer`](WidgetKind::Integer) /
//!   [`Float`](WidgetKind::Float) / [`Text`](WidgetKind::Text)).
//! - Structs, tuples and tuple-structs become a [`Struct`](WidgetKind::Struct)
//!   group with one child per field.
//! - Enums become an [`Enum`](WidgetKind::Enum) whose children are one group per
//!   variant (variant name → its payload fields), so an editor renders a variant
//!   selector plus the selected variant's fields.
//! - Lists/arrays/sets become a [`List`](WidgetKind::List) with a single
//!   element-template child; maps become a [`Map`](WidgetKind::Map) with `key`
//!   and `value` template children.
//! - Anything not further introspectable (opaque types, dynamic fields) is a
//!   read-only [`Opaque`](WidgetKind::Opaque) leaf. Recursion is bounded by
//!   [`MAX_DEPTH`]; a type deeper than that is emitted with `truncated = true`
//!   rather than overflowing the stack on a self-referential type.

use bevy_ecs::prelude::ReflectComponent;
use bevy_reflect::enums::VariantInfo;
use bevy_reflect::{TypeInfo, TypeRegistration, TypeRegistry};
use serde::{Deserialize, Serialize};

/// Recursion ceiling for [`describe_type`]. Real gameplay components nest far
/// shallower than this; the guard exists only so a self-referential type
/// (e.g. a JSON-like enum holding `Vec<Self>`) truncates instead of overflowing
/// the stack.
pub const MAX_DEPTH: usize = 16;

/// Which editor widget renders a reflected field. UI-toolkit agnostic — an egui
/// inspector, a `bevy_ui` panel and the MCP surface all map these to their own
/// controls. Serialized `snake_case` for a stable agent/JSON contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetKind {
    /// A boolean toggle (`bool`).
    Checkbox,
    /// An integer number field (any signed/unsigned integer type).
    Integer,
    /// A floating-point number field (`f32` / `f64`).
    Float,
    /// A single-line text field (`String` / `char` / `str`).
    Text,
    /// A group of named/indexed child fields (struct, tuple, tuple-struct).
    Struct,
    /// A variant selector; `children` holds one [`Struct`](WidgetKind::Struct)
    /// group per variant, in declaration order.
    Enum,
    /// An ordered collection; `children` holds exactly one element-template
    /// descriptor (list, array, set).
    List,
    /// A key→value collection; `children` holds `[key_template, value_template]`.
    Map,
    /// A read-only value that reflection cannot introspect further (opaque or
    /// dynamic type).
    Opaque,
}

/// One node in a reflection-generated widget tree: a field's editor descriptor.
///
/// The root describes a whole type (its `label` and `type_path` are the type's
/// reflect path); children describe its fields recursively. Owned and
/// `serde`-serializable so it can cross the MCP boundary as JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetDescriptor {
    /// Human/agent label: field name, tuple index, `"item"`/`"key"`/`"value"`
    /// for collection templates, variant name for enum variants, or the type
    /// path for a root descriptor.
    pub label: String,
    /// Fully-qualified `bevy_reflect` type path of this node's type.
    pub type_path: String,
    /// The widget that renders this node.
    pub kind: WidgetKind,
    /// Child descriptors — struct fields, enum variant groups, or collection
    /// templates. Empty for leaves.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<WidgetDescriptor>,
    /// Set when recursion hit [`MAX_DEPTH`] and children were omitted — an
    /// honest signal that the tree is incomplete, never a silent cutoff.
    #[serde(default, skip_serializing_if = "is_false")]
    pub truncated: bool,
}

#[allow(clippy::trivially_copy_pass_by_ref)] // signature required by serde's skip_serializing_if
fn is_false(b: &bool) -> bool {
    !*b
}

impl WidgetDescriptor {
    fn leaf(label: String, type_path: &str, kind: WidgetKind) -> Self {
        Self {
            label,
            type_path: type_path.to_owned(),
            kind,
            children: Vec::new(),
            truncated: false,
        }
    }

    fn group(label: String, type_path: &str, kind: WidgetKind, children: Vec<Self>) -> Self {
        Self {
            label,
            type_path: type_path.to_owned(),
            kind,
            children,
            truncated: false,
        }
    }

    fn truncated(label: String, type_path: &str) -> Self {
        Self {
            label,
            type_path: type_path.to_owned(),
            kind: WidgetKind::Opaque,
            children: Vec::new(),
            truncated: true,
        }
    }
}

/// Classify a primitive by its reflect type path. Returns `None` for anything
/// that is not a known scalar, so the caller recurses into its structure.
///
/// Path-based (not `TypeId`-based) so it works uniformly whether or not the
/// field carries static `TypeInfo`. Primitive paths are stable identifiers.
fn leaf_kind(type_path: &str) -> Option<WidgetKind> {
    match type_path {
        "bool" => Some(WidgetKind::Checkbox),
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128"
        | "isize" => Some(WidgetKind::Integer),
        "f32" | "f64" => Some(WidgetKind::Float),
        "char" | "str" | "alloc::string::String" => Some(WidgetKind::Text),
        _ => None,
    }
}

/// Describe a single reflected type as a widget tree. The root's `label` and
/// `type_path` are the type's reflect path.
#[must_use]
pub fn describe_type(info: &TypeInfo) -> WidgetDescriptor {
    describe_node(info.type_path().to_owned(), Some(info), info.type_path(), 0)
}

/// Describe a registered type from its [`TypeRegistration`].
#[must_use]
pub fn describe_registration(registration: &TypeRegistration) -> WidgetDescriptor {
    describe_type(registration.type_info())
}

/// Describe a registered type addressed by reflect type path (the way the editor
/// and MCP name types). `None` if the path is not in the registry.
#[must_use]
pub fn describe_by_path(registry: &TypeRegistry, type_path: &str) -> Option<WidgetDescriptor> {
    registry
        .get_with_type_path(type_path)
        .map(describe_registration)
}

/// Enumerate every registered **component** type as a widget descriptor — the
/// Details-panel surface (the set of components the inspector can edit).
///
/// Sorted by `type_path` so the output is deterministic across runs (the
/// registry's own iteration order is not), which agents rely on for stable
/// diffs.
#[must_use]
pub fn inspect_components(registry: &TypeRegistry) -> Vec<WidgetDescriptor> {
    let mut out: Vec<WidgetDescriptor> = registry
        .iter()
        .filter(|registration| registration.data::<ReflectComponent>().is_some())
        .map(describe_registration)
        .collect();
    out.sort_by(|a, b| a.type_path.cmp(&b.type_path));
    out
}

/// Core recursion. `info` is the field's static type info (if any); `type_path`
/// is always available and is used to classify primitives and label opaque
/// leaves even when `info` is `None` (dynamic fields).
fn describe_node(
    label: String,
    info: Option<&TypeInfo>,
    type_path: &str,
    depth: usize,
) -> WidgetDescriptor {
    // Primitives are terminal regardless of depth.
    if let Some(kind) = leaf_kind(type_path) {
        return WidgetDescriptor::leaf(label, type_path, kind);
    }
    if depth >= MAX_DEPTH {
        return WidgetDescriptor::truncated(label, type_path);
    }
    let Some(info) = info else {
        // Dynamic field with no static type info and not a known primitive.
        return WidgetDescriptor::leaf(label, type_path, WidgetKind::Opaque);
    };
    match info {
        TypeInfo::Struct(s) => {
            let children = s
                .iter()
                .map(|f| {
                    describe_node(f.name().to_owned(), f.type_info(), f.type_path(), depth + 1)
                })
                .collect();
            WidgetDescriptor::group(label, type_path, WidgetKind::Struct, children)
        }
        TypeInfo::TupleStruct(ts) => {
            let children = ts
                .iter()
                .map(|f| {
                    describe_node(
                        f.index().to_string(),
                        f.type_info(),
                        f.type_path(),
                        depth + 1,
                    )
                })
                .collect();
            WidgetDescriptor::group(label, type_path, WidgetKind::Struct, children)
        }
        TypeInfo::Tuple(t) => {
            let children = t
                .iter()
                .map(|f| {
                    describe_node(
                        f.index().to_string(),
                        f.type_info(),
                        f.type_path(),
                        depth + 1,
                    )
                })
                .collect();
            WidgetDescriptor::group(label, type_path, WidgetKind::Struct, children)
        }
        TypeInfo::Enum(e) => {
            let children = e
                .iter()
                .map(|v| describe_variant(v, type_path, depth + 1))
                .collect();
            WidgetDescriptor::group(label, type_path, WidgetKind::Enum, children)
        }
        TypeInfo::List(l) => {
            let item = describe_node(
                "item".to_owned(),
                l.item_info(),
                l.item_ty().path(),
                depth + 1,
            );
            WidgetDescriptor::group(label, type_path, WidgetKind::List, vec![item])
        }
        TypeInfo::Array(a) => {
            let item = describe_node(
                "item".to_owned(),
                a.item_info(),
                a.item_ty().path(),
                depth + 1,
            );
            WidgetDescriptor::group(label, type_path, WidgetKind::List, vec![item])
        }
        TypeInfo::Set(s) => {
            // `SetInfo` exposes the element `Type` but not its `TypeInfo`, so the
            // element is classified by path only (primitive → leaf, else opaque).
            let item = describe_node("item".to_owned(), None, s.value_ty().path(), depth + 1);
            WidgetDescriptor::group(label, type_path, WidgetKind::List, vec![item])
        }
        TypeInfo::Map(m) => {
            let key = describe_node("key".to_owned(), m.key_info(), m.key_ty().path(), depth + 1);
            let value = describe_node(
                "value".to_owned(),
                m.value_info(),
                m.value_ty().path(),
                depth + 1,
            );
            WidgetDescriptor::group(label, type_path, WidgetKind::Map, vec![key, value])
        }
        TypeInfo::Opaque(_) => WidgetDescriptor::leaf(label, type_path, WidgetKind::Opaque),
    }
}

/// Describe one enum variant as a [`Struct`](WidgetKind::Struct) group named for
/// the variant; children are its payload fields (empty for a unit variant).
fn describe_variant(variant: &VariantInfo, enum_path: &str, depth: usize) -> WidgetDescriptor {
    let name = variant.name();
    let variant_path = format!("{enum_path}::{name}");
    let children = match variant {
        VariantInfo::Struct(sv) => sv
            .iter()
            .map(|f| describe_node(f.name().to_owned(), f.type_info(), f.type_path(), depth + 1))
            .collect(),
        VariantInfo::Tuple(tv) => tv
            .iter()
            .map(|f| {
                describe_node(
                    f.index().to_string(),
                    f.type_info(),
                    f.type_path(),
                    depth + 1,
                )
            })
            .collect(),
        VariantInfo::Unit(_) => Vec::new(),
    };
    WidgetDescriptor::group(name.to_owned(), &variant_path, WidgetKind::Struct, children)
}

#[cfg(test)]
mod tests;
