//! Netcode: move authoritative state to many nearby clients at the tick rate,
//! within a bandwidth budget, over a lossy link, without ever trusting the client.
//!
//! This crate is **pure** — no sockets. The socket/transport wiring lives in
//! `apps/shard` / `apps/gateway`; here we keep the algorithms that must be
//! exhaustively, deterministically tested (docs/specs/game-server/netcode).
//!
//! The pipeline, in the order a shard applies it each tick:
//! 1. [`aoi`] — filter the world to a client's area of interest, so bandwidth is
//!    `O(nearby)`, not `O(world)`.
//! 2. [`priority`] — fill a per-client byte budget by priority
//!    (self > combat > near > far); low-priority entities skip ticks when over.
//! 3. [`delta`] — send only what changed versus the last snapshot that client
//!    acked; a dropped snapshot just deltas against an older baseline (Quake3).
//! 4. [`quantize`] — mm-quantize positions and pack angles before send.
//! 5. [`reliability`] — sequence/ack math so reliable channels survive loss.
//!
//! Under all of that sits [`transport`]: a runtime-agnostic [`Transport`] trait
//! and the length-prefixed [`Frame`] header (seq/ack/ack-bits) that carries the
//! reliability state on the wire. The trait is the seam a loopback (tests) or a
//! UDP socket (prod) plugs into; the crate itself keeps only the framing so it
//! stays deterministically testable.

pub mod aoi;
pub mod delta;
pub mod priority;
pub mod quantize;
pub mod reliability;
pub mod snapshot;
pub mod transport;

pub use aoi::filter_by_interest;
pub use delta::{apply_delta, diff, DeltaFrame};
pub use priority::{fill_budget, Candidate, Priority};
pub use quantize::{dequantize_angle, dequantize_pos, quantize_angle, quantize_pos};
pub use reliability::{seq_greater_than, AckTracker};
pub use snapshot::{EntitySnapshot, SnapshotFrame};
pub use transport::{
    framing::{Frame, FramingError},
    loopback::{Loopback, LossModel},
    ConnId, Transport, TransportError,
};
