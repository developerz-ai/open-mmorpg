//! The datagram transport seam: a runtime-agnostic [`Transport`] trait plus the
//! [`framing`] header that reliability rides on.
//!
//! A [`Transport`] is one logical connection (client↔shard). It moves opaque
//! datagrams and nothing else — best-effort, unordered, no redelivery. Ordering,
//! acks and resends are the reliability layer's job ([`crate::reliability`]),
//! carried in each [`framing::Frame`]. Keeping the trait this thin lets us swap
//! an in-memory loopback (tests, single-process) for a real UDP socket (prod)
//! without touching a line of gameplay code.

pub mod framing;
pub mod loopback;

pub use framing::{Frame, FramingError};
pub use loopback::{Loopback, LossModel};

/// Identifies one logical connection (client↔shard). Minted at accept time; the
/// server tags per-connection reliability state by it. Never a raw `u64` — the
/// type system keeps a connection id from being confused with any other id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConnId(pub u64);

impl ConnId {
    /// Wrap a raw connection id. Prefer minting at the accept boundary.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The underlying raw value — use only at storage/wire edges.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for ConnId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ConnId({})", self.0)
    }
}

/// A transport-level failure. Detail is kept generic and server-side safe: no
/// peer addresses, no payload bytes, nothing a client could weaponize.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// The connection is closed; no further datagrams can flow either way.
    #[error("connection {0} closed")]
    Closed(ConnId),
    /// A datagram exceeded the transport's maximum on-wire size.
    #[error("datagram too large: {size} bytes (max {max})")]
    TooLarge {
        /// The rejected datagram's size in bytes.
        size: usize,
        /// The transport's maximum datagram size in bytes.
        max: usize,
    },
    /// A received buffer could not be parsed into a [`Frame`].
    #[error("framing: {0}")]
    Framing(#[from] FramingError),
    /// The underlying socket failed. Only the generic error *kind* is retained —
    /// never OS strings that might carry paths or addresses.
    #[error("transport i/o: {0}")]
    Io(std::io::ErrorKind),
}

impl From<std::io::Error> for TransportError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.kind())
    }
}

/// One logical connection over which datagrams flow. Concrete impls (loopback,
/// UDP) live behind this trait so the shard can hold `Arc<dyn Transport>` and
/// swap them freely; `Send + Sync` lets a sender and receiver task share one.
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// The connection this transport serves.
    fn conn_id(&self) -> ConnId;

    /// Send one datagram to the peer. Delivery is best-effort — a UDP transport
    /// may silently drop it; redelivery and ordering are the reliability layer's
    /// concern, never the transport's.
    async fn send(&self, dgram: &[u8]) -> Result<(), TransportError>;

    /// Await the next datagram from the peer. Returns raw bytes; framing and ack
    /// bookkeeping happen a layer up in [`framing`] / [`crate::reliability`].
    async fn recv(&self) -> Result<Vec<u8>, TransportError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conn_id_roundtrips_and_displays() {
        let id = ConnId::new(42);
        assert_eq!(id.raw(), 42);
        assert_eq!(id, ConnId(42));
        assert_eq!(id.to_string(), "ConnId(42)");
    }

    #[test]
    fn io_error_keeps_only_the_kind() {
        let io = std::io::Error::new(std::io::ErrorKind::ConnectionReset, "secret-host:1234");
        let err: TransportError = io.into();
        match err {
            TransportError::Io(kind) => assert_eq!(kind, std::io::ErrorKind::ConnectionReset),
            other => panic!("expected Io, got {other:?}"),
        }
        // The leaky detail string must not survive into the public message.
        assert!(
            !err_to_string(&TransportError::Io(std::io::ErrorKind::ConnectionReset))
                .contains("secret-host")
        );
    }

    #[test]
    fn closed_error_names_the_connection() {
        let err = TransportError::Closed(ConnId::new(7));
        assert_eq!(err.to_string(), "connection ConnId(7) closed");
    }

    fn err_to_string(e: &TransportError) -> String {
        e.to_string()
    }
}
