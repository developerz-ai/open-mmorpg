//! [`Module`] — what a compiled module *is* to the host.
//!
//! A module is a [`ServerHooks`] implementation plus a [`ModuleManifest`]
//! identity, held by the [`crate::ModuleHost`] as a `Box<dyn Module>`. The trait
//! is object-safe (so the host can store a heterogeneous set) and downcastable
//! via [`Module::as_any`] (so tooling — and tests — can recover a concrete
//! module to inspect its state).

use std::any::Any;

use crate::hooks::ServerHooks;
use crate::manifest::ModuleManifest;

/// A self-contained unit of gameplay a fork links into the server.
///
/// Implement [`ServerHooks`] for the events you handle, then `impl Module` to
/// declare identity and enable downcasting. The [`crate::declare_module!`] macro
/// emits the `module()` entry point the generated registry calls to construct
/// one — a module never registers itself by editing a core list.
pub trait Module: ServerHooks + Any {
    /// This module's runtime identity, for logs and operator tooling.
    fn manifest(&self) -> ModuleManifest;

    /// Upcast to [`Any`] so the host can downcast to a concrete module type.
    ///
    /// The one-line body is always `self`; the scaffold writes it for you. It
    /// lets [`crate::ModuleHost::get`] hand back a `&ConcreteModule` — used by
    /// inspection tooling and by tests asserting a real event reached a real,
    /// linked-in module.
    fn as_any(&self) -> &dyn Any;
}
