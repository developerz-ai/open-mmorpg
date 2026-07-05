//! Host for **untrusted operator content scripts** — abilities, quest logic,
//! reactive NPC/faction AI.
//!
//! Operator scripts are untrusted by definition (anyone can ship a datapack), so
//! they run sandboxed with an explicit, capability-limited surface:
//!
//! - [`wasm`] — the primary target: a `wasmtime` host that compiles module bytes
//!   and runs an entry function under a **fuel budget**, with only the
//!   [`capability`] API linked. A runaway script is starved
//!   ([`error::ScriptError::OutOfFuel`]), never able to hang a shard, and it can
//!   reach no filesystem, network, cache, clock, or database.
//! - [`capability`] — the narrow [`HostCapabilities`] trait, the *entire* thing a
//!   script can do. Ownership writes never happen inside the sandbox; a grant is
//!   only expressed as intent and committed by the host through `omm-persistence`
//!   in a single transaction.
//! - [`registry`] — a deterministic in-process reference engine used to exercise
//!   the host contract before real content is authored.
//!
//! See `docs/architecture/05-ecs-and-scripting.md` and the security spec.

pub mod capability;
pub mod error;
pub mod registry;
pub mod wasm;

pub use capability::{EffectId, EntityId, HostCall, HostCapabilities, RecordingHost};
pub use error::{ScriptError, ScriptResult};
pub use registry::{Registry, ScriptCtx, ScriptEngine};
pub use wasm::{CompiledScript, Fuel, Outcome, WasmHost};

use omm_content_schema::Manifest;
use omm_errors::CoreResult;

/// Bind a validated datapack into a script engine. In the scaffold this only
/// confirms the manifest is loadable; the real loader will resolve and compile
/// each script referenced by the pack. Returns the number of factions bound as a
/// stand-in for "content wired up".
///
/// # Errors
/// Propagates validation errors from the content layer.
pub fn load_datapack(manifest: &Manifest) -> CoreResult<usize> {
    omm_content_schema::validate(manifest)?;
    Ok(manifest.factions.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datapack_binds_after_validation() {
        let m = Manifest {
            id: "test.pack".into(),
            version: "0.0.0".into(),
            api_version: omm_content_schema::CONTENT_API_VERSION,
            factions: vec![],
            races: vec![],
            classes: vec![],
            abilities: vec![],
            items: vec![],
            quests: vec![],
            zones: vec![],
            spawn_tables: vec![],
            dungeons: vec![],
            economy: Default::default(),
            asset_manifest_ref: None,
        };
        assert_eq!(load_datapack(&m).unwrap(), 0);
    }
}
