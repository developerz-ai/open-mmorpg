//! `omm-shard` — the authoritative zone server's reusable core.
//!
//! The `omm-shard` binary ([`main`](../main.rs)) is a thin tokio entrypoint over
//! these modules:
//! - [`tick`] — the fixed-timestep pacer that decouples wall-clock from sim
//!   ticks, so `World::step` always sees the same `dt` (the property replay and
//!   anti-cheat re-simulation depend on).
//! - [`session`] — the registry binding each live connection to the one
//!   server-issued entity it drives, the seam that keeps the server
//!   authoritative over identity.
//!
//! Splitting these into a library (rather than burying them in `main`) is what
//! lets every lifecycle edge be unit-tested without a socket or a running loop.

pub mod session;
pub mod tick;
