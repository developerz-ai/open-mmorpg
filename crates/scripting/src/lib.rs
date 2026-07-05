//! Host for **untrusted operator content scripts** — abilities, quest logic,
//! reactive NPC/faction AI.
//!
//! Operator scripts are untrusted by definition (anyone can ship a datapack), so
//! they run sandboxed with an explicit, capability-limited surface. The engine
//! is pluggable behind [`ScriptEngine`]: WASM (`wasmtime`) is the primary target
//! and Lua (`mlua`) the lightweight one; this scaffold ships a deterministic
//! in-process [`Registry`] so the host contract and the mod-loading flow are
//! testable now (docs/architecture/05-ecs-and-scripting.md).

use omm_content_schema::Manifest;
use omm_errors::{CoreError, CoreResult};
use std::collections::HashMap;

/// The read-only context a script is given. Scripts see only what we pass in —
/// no ambient filesystem, network, or clock. This is the sandbox surface.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScriptCtx {
    /// Caller-provided argument (e.g. ability rank, quest step). Placeholder for
    /// the richer typed context the WASM ABI will expose.
    pub arg: i64,
}

/// A hosted script engine. Implementations must be deterministic given the same
/// script id and context, so scripted outcomes are replayable and re-verifiable.
pub trait ScriptEngine {
    /// Invoke script `id` with `ctx`, returning its integer result.
    ///
    /// # Errors
    /// [`CoreError::NotFound`] if no script is registered under `id`.
    fn invoke(&self, id: &str, ctx: ScriptCtx) -> CoreResult<i64>;
}

/// Boxed native handler used by the in-process reference engine.
type Handler = Box<dyn Fn(ScriptCtx) -> i64 + Send + Sync>;

/// In-process reference engine: named native handlers stand in for compiled
/// WASM modules so the rest of the stack can call scripts before the real VM
/// lands. Deterministic and side-effect free.
#[derive(Default)]
pub struct Registry {
    handlers: HashMap<String, Handler>,
}

impl Registry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a script handler under `id`.
    pub fn register<F>(&mut self, id: &str, handler: F)
    where
        F: Fn(ScriptCtx) -> i64 + Send + Sync + 'static,
    {
        self.handlers.insert(id.to_owned(), Box::new(handler));
    }

    /// Number of registered scripts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Whether no scripts are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

impl ScriptEngine for Registry {
    fn invoke(&self, id: &str, ctx: ScriptCtx) -> CoreResult<i64> {
        self.handlers
            .get(id)
            .map(|h| h(ctx))
            .ok_or_else(|| CoreError::NotFound(format!("script '{id}'")))
    }
}

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
    fn registered_script_runs_deterministically() {
        let mut reg = Registry::new();
        reg.register("ability.firebolt", |ctx| ctx.arg * 3 + 10);
        assert!(!reg.is_empty());
        assert_eq!(reg.len(), 1);

        let a = reg
            .invoke("ability.firebolt", ScriptCtx { arg: 4 })
            .unwrap();
        let b = reg
            .invoke("ability.firebolt", ScriptCtx { arg: 4 })
            .unwrap();
        assert_eq!(a, 22);
        assert_eq!(a, b);
    }

    #[test]
    fn unknown_script_is_not_found() {
        let reg = Registry::new();
        let err = reg.invoke("missing", ScriptCtx::default()).unwrap_err();
        assert_eq!(err.code(), omm_errors::ClientCode::NotFound);
    }

    #[test]
    fn datapack_binds_after_validation() {
        let m = Manifest {
            id: "test.pack".into(),
            version: "0.0.0".into(),
            api_version: omm_content_schema::CONTENT_API_VERSION,
            factions: vec![],
        };
        assert_eq!(load_datapack(&m).unwrap(), 0);
    }
}
