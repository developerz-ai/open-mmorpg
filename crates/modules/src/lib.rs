//! `omm-modules` — the generated module aggregator the server loads.
//!
//! This crate is deliberately tiny: its [`build.rs`](../build.rs) scans
//! `modules/` and generates `register_all`, and [`load`] runs it into a fresh
//! [`ModuleHost`]. A server binary depends on this one crate and calls [`load`]
//! once at boot; every module under `modules/` is then linked in and dispatched,
//! with no core file naming any individual module.
//!
//! The `omm-module-*` path dependencies in `Cargo.toml` are what actually *link*
//! each module crate into the binary (Cargo drops crates nothing references, so
//! naming them as deps and calling their `module()` is the force-link). Those
//! dep lines are managed by `bin/new-module`; the generated `register_all` calls
//! into them.

use omm_module_api::ModuleHost;

// The generated `register_all(&mut ModuleHost)` — one `host.register(...)` per
// module crate under `modules/`, in deterministic sorted order.
include!(concat!(env!("OUT_DIR"), "/registrations.rs"));

/// Build a [`ModuleHost`] with every compiled-in module registered.
///
/// Call once at server boot and hold the returned host for the process's life:
/// the core dispatches each server event through it. An empty `modules/` yields
/// an empty host — a valid no-op, not an error.
#[must_use]
pub fn load() -> ModuleHost {
    let mut host = ModuleHost::new();
    register_all(&mut host);
    host
}
