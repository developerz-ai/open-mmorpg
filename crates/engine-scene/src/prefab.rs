//! Prefabs — reusable **scene fragments** spawned into the world from data.
//!
//! A prefab is an ordered list of entities, each an ordered list of *reflected*
//! components. [`Prefab::spawn`] is the deterministic spawn-from-data primitive
//! (the reflection-driven successor to Bevy ≤0.18's `DynamicScene::write_to_world`,
//! which the 0.19 BSN rework removed): it walks entities and components in
//! declared order, so a headless run reproduces a headful one bit-for-bit
//! (→ `docs/specs/game-engine/scene/README.md`).
//!
//! Reflection is the whole contract. A component type absent from the registry
//! is invisible to scenes, the inspector and agents, so spawning refuses it
//! **before** touching the world ([`SceneError::UnregisteredType`]); a type that
//! is registered but not a spawnable `Component` aborts the spawn and rolls back
//! ([`SceneError::PartialSpawn`]) — never a silent half-spawned world.

use core::fmt;

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{AppTypeRegistry, ReflectComponent};
use bevy_ecs::world::World;
use bevy_reflect::PartialReflect;

use crate::error::SceneError;

/// One entity's worth of reflected components, kept in insertion order.
#[derive(Default)]
pub struct EntityBlueprint {
    components: Vec<Box<dyn PartialReflect>>,
}

impl EntityBlueprint {
    /// A blueprint with no components.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a typed component. Order is preserved on spawn.
    #[must_use]
    pub fn with<C: PartialReflect>(mut self, component: C) -> Self {
        self.components.push(Box::new(component));
        self
    }

    /// Append an already-reflected component — the path the RON loader and
    /// reflection-based tools use, where the concrete type is only known
    /// dynamically.
    #[must_use]
    pub fn with_boxed(mut self, component: Box<dyn PartialReflect>) -> Self {
        self.components.push(component);
        self
    }

    /// Number of components on this entity.
    #[must_use]
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Whether this entity carries no components.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub(crate) fn components(&self) -> &[Box<dyn PartialReflect>] {
        &self.components
    }
}

/// A scene fragment: an ordered list of entities to spawn together.
#[derive(Default)]
pub struct Prefab {
    entities: Vec<EntityBlueprint>,
}

impl Prefab {
    /// An empty prefab.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an entity blueprint. Entities spawn in the order added.
    #[must_use]
    pub fn with_entity(mut self, entity: EntityBlueprint) -> Self {
        self.entities.push(entity);
        self
    }

    /// Number of entities this prefab will spawn.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Whether the prefab spawns no entities.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// The entity blueprints, in declared order.
    #[must_use]
    pub fn entities(&self) -> &[EntityBlueprint] {
        &self.entities
    }

    /// Spawn this prefab into `world`, returning the created entities in
    /// declared order.
    ///
    /// Deterministic: entities and their components are applied in the order they
    /// were added, so the same prefab spawned into the same world state yields
    /// identical [`Entity`] ids every time.
    ///
    /// # Errors
    /// - [`SceneError::UnregisteredType`] if any component type is missing from
    ///   the reflection registry — checked up front, so nothing is spawned.
    /// - [`SceneError::PartialSpawn`] if a component is registered but not a
    ///   spawnable `Component`; every entity created so far is despawned so the
    ///   world is left exactly as it was found.
    pub fn spawn(&self, world: &mut World) -> Result<Vec<Entity>, SceneError> {
        // Clone the registry handle (an `Arc`) out of the world so we can hold a
        // read lock while mutating the world through it.
        let type_registry = world.resource::<AppTypeRegistry>().clone();
        let registry = type_registry.read();

        // Pass 1 — every component type must be registered. Reject atomically,
        // before the world is touched, so invalid data spawns nothing.
        for entity in &self.entities {
            for component in entity.components() {
                let type_path = represented_type_path(component.as_ref());
                if registry.get_with_type_path(type_path).is_none() {
                    return Err(SceneError::UnregisteredType {
                        type_path: type_path.to_owned(),
                    });
                }
            }
        }

        // Pass 2 — spawn in declared order. A type that is registered but not a
        // `Component` is only detectable here; roll back the whole fragment so
        // the world never keeps a partial scene.
        let mut spawned: Vec<Entity> = Vec::with_capacity(self.entities.len());
        for entity in &self.entities {
            let mut entity_mut = world.spawn_empty();
            let id = entity_mut.id();
            for component in entity.components() {
                let type_path = represented_type_path(component.as_ref());
                let reflect_component = registry
                    .get_with_type_path(type_path)
                    .and_then(|registration| registration.data::<ReflectComponent>());
                match reflect_component {
                    Some(reflect_component) => {
                        reflect_component.insert(&mut entity_mut, component.as_ref(), &registry);
                    }
                    None => {
                        let type_path = type_path.to_owned();
                        let committed = spawned.len();
                        // Despawn the in-progress entity (consuming its borrow),
                        // then every entity committed earlier — full rollback.
                        entity_mut.despawn();
                        for &entity in &spawned {
                            let _ = world.despawn(entity);
                        }
                        return Err(SceneError::PartialSpawn {
                            spawned: committed,
                            type_path,
                        });
                    }
                }
            }
            spawned.push(id);
        }
        Ok(spawned)
    }
}

/// Reflection type path of a component value, or `<unknown>` for a pure dynamic
/// value that carries no represented type (which then fails the registry check).
fn represented_type_path(component: &dyn PartialReflect) -> &'static str {
    component
        .get_represented_type_info()
        .map_or("<unknown>", |info| info.type_path())
}

// `Box<dyn PartialReflect>` has no `Debug`, so print the structure by component
// type path — enough for logs and agents to see what a prefab will spawn.
impl fmt::Debug for EntityBlueprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(
                self.components
                    .iter()
                    .map(|component| represented_type_path(component.as_ref())),
            )
            .finish()
    }
}

impl fmt::Debug for Prefab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Prefab")
            .field("entities", &self.entities)
            .finish()
    }
}

#[cfg(test)]
mod tests;
