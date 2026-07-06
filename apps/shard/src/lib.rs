//! `omm-shard` — the authoritative zone server's reusable core.
//!
//! The `omm-shard` binary ([`main`](../main.rs)) is a thin tokio entrypoint over
//! these modules:
//! - [`abilities`] — the data→runtime bridge that lowers authored
//!   `content-schema` abilities onto the compiled `ecs-core` shape at boot,
//!   fail-loud on invalid data.
//! - [`tick`] — the fixed-timestep pacer that decouples wall-clock from sim
//!   ticks, so `World::step` always sees the same `dt` (the property replay and
//!   anti-cheat re-simulation depend on).
//! - [`session`] — the registry binding each live connection to the one
//!   server-issued entity it drives, the seam that keeps the server
//!   authoritative over identity.
//! - [`replicate`] — the snapshot egress: per client, filter the world to its
//!   area of interest, fill a bandwidth budget by priority, and delta against
//!   its acked baseline before sending an authoritative snapshot.
//!
//! Splitting these into a library (rather than burying them in `main`) is what
//! lets every lifecycle edge be unit-tested without a socket or a running loop.

pub mod abilities;
pub mod replicate;
pub mod session;
pub mod tick;
