//! `omm-transport`: where netcode bytes actually hit the wire.
//!
//! [`omm_netcode`] stays **pure** — framing and reliability math, no sockets. This
//! crate is the impure edge that plugs into its [`Transport`] seam:
//!
//! - [`udp::UdpTransport`] — a `tokio::net::UdpSocket` [`Transport`], one connected
//!   socket per logical connection. Best-effort datagrams in, best-effort out;
//!   ordering/acks/resends stay a layer up in [`omm_netcode::reliability`].
//! - [`connection::Connection`] — the per-connection **lifecycle** state machine a
//!   shard drives above the transport: `Hello` → accept/reject, `Ping`/`Pong`
//!   keepalive, and idle/handshake timeouts. It is a pure, clock-free state
//!   machine ([`ConnState`] makes illegal phases unrepresentable), so the whole
//!   lifecycle is deterministically unit-tested without a socket or a wall clock.
//!
//! The split mirrors the rest of the stack: transport moves opaque bytes, the
//! lifecycle decides *whether* a connection may speak, and gameplay lives above
//! both. Swapping loopback (tests) for UDP (prod) touches no gameplay code.

pub mod connection;
pub mod udp;

pub use connection::{CloseReason, ConnEvent, ConnState, Connection, Timeouts};
pub use udp::{UdpTransport, DEFAULT_MAX_DATAGRAM};

// Re-export the seam so downstream crates lean on one transport crate, not two.
pub use omm_netcode::transport::{ConnId, Transport, TransportError};
