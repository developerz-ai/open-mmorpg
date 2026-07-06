//! Typed errors for scene/prefab loading and spawning.
//!
//! Scene data is authored by hand or by an agent, so it *will* be wrong
//! sometimes; the contract (see `docs/specs/game-engine/scene/README.md`) is that
//! invalid data fails **loud at load** and never leaves a silent half-spawned
//! world. Each variant names a distinct, actionable failure so a tool or agent
//! can react precisely rather than parsing a string.

use thiserror::Error;

/// Everything that can go wrong turning scene data into live entities.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SceneError {
    /// The prefab text could not be parsed into a scene — malformed RON, or a
    /// component whose type/field data the reflection deserializer rejected.
    #[error("failed to parse prefab: {0}")]
    Parse(String),

    /// A component names a type absent from the reflection registry. An
    /// unregistered type is invisible to scenes, the inspector and agents — that
    /// is a bug, not a warning, so spawning refuses it before touching the world.
    #[error("component type `{type_path}` is not registered for reflection")]
    UnregisteredType {
        /// Reflection type path of the offending component
        /// (e.g. `bevy_transform::components::transform::Transform`).
        type_path: String,
    },

    /// A component's type was registered but is not usable as a `Component`
    /// (e.g. registered without `#[reflect(Component)]`), discovered only after
    /// earlier entities had been created. The partial spawn is rolled back so the
    /// world is left untouched — "never a silent partial spawn".
    #[error(
        "prefab spawn rolled back: {spawned} entities were created before component \
         `{type_path}` (not a spawnable Component) aborted the spawn"
    )]
    PartialSpawn {
        /// Number of entities that had been created before the abort (all since
        /// despawned).
        spawned: usize,
        /// Reflection type path of the component that could not be applied.
        type_path: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_reads_cleanly() {
        let err = SceneError::Parse("unexpected `}`".to_owned());
        assert_eq!(err.to_string(), "failed to parse prefab: unexpected `}`");
    }

    #[test]
    fn unregistered_type_names_the_type() {
        let err = SceneError::UnregisteredType {
            type_path: "game::Health".to_owned(),
        };
        assert!(err.to_string().contains("game::Health"));
        assert!(err.to_string().contains("not registered"));
    }

    #[test]
    fn partial_spawn_reports_count_and_names_the_type() {
        let err = SceneError::PartialSpawn {
            spawned: 3,
            type_path: "game::Marker".to_owned(),
        };
        let msg = err.to_string();
        assert!(msg.contains("3 entities"), "got: {msg}");
        assert!(msg.contains("game::Marker"), "got: {msg}");
        assert!(msg.contains("rolled back"), "got: {msg}");
    }
}
