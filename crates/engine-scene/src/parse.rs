//! Text authoring for prefabs — parse RON into a [`Prefab`] through the
//! reflection registry, and serialize a [`Prefab`] back to RON.
//!
//! A prefab is written as a sequence of entities, each a sequence of components;
//! every component is a single-entry map keyed by its reflection type path (the
//! shape [`ReflectDeserializer`] consumes and [`ReflectSerializer`] produces). No
//! per-type serializer is written by hand: `#[derive(Reflect)]` + a registry
//! entry is the whole contract, which is exactly why an agent can author a valid
//! prefab and why an invalid one fails **loud** here rather than silently at
//! spawn (→ scene spec).
//!
//! ```ron
//! [
//!     [
//!         {"my_game::Health": (current: 30.0, max: 100.0)},
//!         {"my_game::Level": (value: 3)},
//!     ],
//! ]
//! ```

use core::fmt;

use bevy_reflect::serde::{ReflectDeserializer, ReflectSerializer};
use bevy_reflect::TypeRegistry;
use ron::Options;
use serde::de::{DeserializeSeed, Deserializer, SeqAccess, Visitor};

use crate::error::SceneError;
use crate::prefab::{EntityBlueprint, Prefab};

impl Prefab {
    /// Parse a RON prefab against `registry`.
    ///
    /// # Errors
    /// [`SceneError::Parse`] if the text is malformed or names a component type
    /// the registry does not know — invalid data never yields a half-built
    /// prefab.
    pub fn from_ron(ron: &str, registry: &TypeRegistry) -> Result<Prefab, SceneError> {
        Options::default()
            .from_str_seed(ron, PrefabSeed { registry })
            .map_err(|err| SceneError::Parse(err.to_string()))
    }

    /// Serialize this prefab back to a RON string using the reflection registry.
    ///
    /// The output uses exactly the same `{"TypePath": value}` per-component format
    /// that [`Self::from_ron`] expects, so
    /// `Prefab::from_ron(&prefab.to_ron(r)?, r)?` reproduces equivalent component
    /// values — the round-trip contract.
    ///
    /// # Errors
    /// [`SceneError::Parse`] if any component cannot be serialized (e.g. its
    /// concrete type is unrepresented in the registry).
    pub fn to_ron(&self, registry: &TypeRegistry) -> Result<String, SceneError> {
        let mut entities_ron: Vec<String> = Vec::with_capacity(self.entities().len());
        for entity in self.entities() {
            let mut components_ron: Vec<String> = Vec::with_capacity(entity.len());
            for component in entity.components() {
                let serializer = ReflectSerializer::new(component.as_ref(), registry);
                let s = ron::to_string(&serializer)
                    .map_err(|e| SceneError::Parse(format!("serialize: {e}")))?;
                components_ron.push(s);
            }
            entities_ron.push(format!("[{}]", components_ron.join(",")));
        }
        Ok(format!("[{}]", entities_ron.join(",")))
    }
}

/// Seed that deserializes the whole prefab (a sequence of entities).
struct PrefabSeed<'a> {
    registry: &'a TypeRegistry,
}

impl<'de> DeserializeSeed<'de> for PrefabSeed<'_> {
    type Value = Prefab;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(PrefabVisitor {
            registry: self.registry,
        })
    }
}

struct PrefabVisitor<'a> {
    registry: &'a TypeRegistry,
}

impl<'de> Visitor<'de> for PrefabVisitor<'_> {
    type Value = Prefab;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of prefab entities")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut prefab = Prefab::new();
        while let Some(entity) = seq.next_element_seed(EntitySeed {
            registry: self.registry,
        })? {
            prefab = prefab.with_entity(entity);
        }
        Ok(prefab)
    }
}

/// Seed that deserializes one entity (a sequence of reflected components).
struct EntitySeed<'a> {
    registry: &'a TypeRegistry,
}

impl<'de> DeserializeSeed<'de> for EntitySeed<'_> {
    type Value = EntityBlueprint;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(EntityVisitor {
            registry: self.registry,
        })
    }
}

struct EntityVisitor<'a> {
    registry: &'a TypeRegistry,
}

impl<'de> Visitor<'de> for EntityVisitor<'_> {
    type Value = EntityBlueprint;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of reflected components")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut entity = EntityBlueprint::new();
        // Each element is a `{ "type::path": value }` map — exactly what the
        // reflection deserializer consumes, so the registry is the only schema.
        while let Some(component) =
            seq.next_element_seed(ReflectDeserializer::new(self.registry))?
        {
            entity = entity.with_boxed(component);
        }
        Ok(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_ecs::prelude::*;
    use bevy_reflect::Reflect;

    #[derive(Component, Reflect, Default, PartialEq, Debug)]
    #[reflect(Component)]
    struct Health {
        current: f32,
        max: f32,
    }

    #[derive(Component, Reflect, Default, PartialEq, Debug)]
    #[reflect(Component)]
    struct Level {
        value: u32,
    }

    fn registry_app() -> App {
        let mut app = App::new();
        app.register_type::<Health>();
        app.register_type::<Level>();
        app
    }

    #[test]
    fn from_ron_loads_and_spawns() {
        let mut app = registry_app();
        let ron = r#"[
            [
                {"omm_engine_scene::parse::tests::Health": (current: 30.0, max: 100.0)},
                {"omm_engine_scene::parse::tests::Level": (value: 3)},
            ],
        ]"#;

        let prefab = {
            let registry = app.world().resource::<AppTypeRegistry>().clone();
            let registry = registry.read();
            match Prefab::from_ron(ron, &registry) {
                Ok(prefab) => prefab,
                Err(err) => panic!("from_ron failed: {err}"),
            }
        };
        assert_eq!(prefab.len(), 1);

        let spawned = match prefab.spawn(app.world_mut()) {
            Ok(entities) => entities,
            Err(err) => panic!("spawn failed: {err}"),
        };
        let Some(&entity) = spawned.first() else {
            panic!("no entity spawned")
        };
        assert_eq!(
            app.world().get::<Health>(entity),
            Some(&Health {
                current: 30.0,
                max: 100.0
            })
        );
        assert_eq!(app.world().get::<Level>(entity), Some(&Level { value: 3 }));
    }

    #[test]
    fn from_ron_malformed_text_is_parse_error() {
        let app = registry_app();
        let registry = app.world().resource::<AppTypeRegistry>().clone();
        let registry = registry.read();
        match Prefab::from_ron("this is not ron {{{", &registry) {
            Err(SceneError::Parse(_)) => {}
            other => panic!("expected Parse, got {other:?}"),
        }
    }

    #[test]
    fn from_ron_unknown_type_is_parse_error() {
        let app = registry_app();
        let registry = app.world().resource::<AppTypeRegistry>().clone();
        let registry = registry.read();
        let ron = r#"[[{"game::DoesNotExist": ()}]]"#;
        match Prefab::from_ron(ron, &registry) {
            Err(SceneError::Parse(_)) => {}
            other => panic!("expected Parse for an unknown type, got {other:?}"),
        }
    }
}
