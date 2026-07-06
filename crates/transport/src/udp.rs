//! A real [`Transport`] over a `tokio::net::UdpSocket`.
//!
//! One [`UdpTransport`] is one logical connection: a socket bound locally and
//! *connected* to a single peer, so [`Transport::send`]/[`Transport::recv`] carry
//! no address and behave exactly like the in-memory loopback the tests use. A
//! shard swaps one for the other with no gameplay change.
//!
//! Datagrams are moved whole and opaque — one datagram carries one
//! [`omm_netcode::Frame`], and framing/acks/resends stay a layer up. Sends are
//! capped at [`DEFAULT_MAX_DATAGRAM`]: game netcode keeps every datagram inside
//! one un-fragmented UDP packet on purpose, because a lost IP fragment loses the
//! whole datagram. Oversize sends fail loudly with [`TransportError::TooLarge`]
//! rather than fragmenting on the wire.

use std::net::SocketAddr;

use tokio::net::UdpSocket;

use omm_netcode::transport::{ConnId, Transport, TransportError};

/// Default maximum on-wire datagram size, in bytes.
///
/// 1200 is the fragmentation-safe payload every common path MTU (incl. IPv6's
/// 1280 minus headers, and the QUIC/WebTransport default) carries without IP
/// fragmentation. Larger reliable messages are the reliability layer's job to
/// split — never the transport's to fragment.
pub const DEFAULT_MAX_DATAGRAM: usize = 1200;

/// A [`Transport`] backed by a connected UDP socket serving exactly one peer.
///
/// `tokio::net::UdpSocket` sends and receives through `&self`, so one instance is
/// shared by a sender task and a receiver task behind an `Arc<dyn Transport>`
/// with no lock on the tick path — same shape as the loopback.
#[derive(Debug)]
pub struct UdpTransport {
    conn: ConnId,
    socket: UdpSocket,
    max_datagram: usize,
}

impl UdpTransport {
    /// Bind `local` and connect the socket to `peer` — the one peer this
    /// transport serves. A connected socket drops datagrams from any other
    /// source, which is the isolation a per-connection [`Transport`] needs.
    ///
    /// Pass `local` with port `0` to let the OS pick a free port; read it back
    /// with [`UdpTransport::local_addr`].
    ///
    /// # Errors
    /// [`TransportError::Io`] if the bind or connect fails (address in use,
    /// permission denied, unreachable peer). Only the error *kind* is retained —
    /// never an OS string that might carry a host or path.
    pub async fn connect(
        conn: ConnId,
        local: SocketAddr,
        peer: SocketAddr,
    ) -> Result<Self, TransportError> {
        let socket = UdpSocket::bind(local).await?;
        socket.connect(peer).await?;
        Ok(Self {
            conn,
            socket,
            max_datagram: DEFAULT_MAX_DATAGRAM,
        })
    }

    /// The local address actually bound — resolves the OS-assigned port when
    /// [`UdpTransport::connect`] was given port `0`.
    ///
    /// # Errors
    /// [`TransportError::Io`] if the socket has no local address.
    pub fn local_addr(&self) -> Result<SocketAddr, TransportError> {
        Ok(self.socket.local_addr()?)
    }

    /// The connected peer address.
    ///
    /// # Errors
    /// [`TransportError::Io`] if the socket is not connected.
    pub fn peer_addr(&self) -> Result<SocketAddr, TransportError> {
        Ok(self.socket.peer_addr()?)
    }

    /// The largest datagram this transport will send, in bytes.
    #[must_use]
    pub const fn max_datagram(&self) -> usize {
        self.max_datagram
    }
}

#[async_trait::async_trait]
impl Transport for UdpTransport {
    fn conn_id(&self) -> ConnId {
        self.conn
    }

    async fn send(&self, dgram: &[u8]) -> Result<(), TransportError> {
        if dgram.len() > self.max_datagram {
            return Err(TransportError::TooLarge {
                size: dgram.len(),
                max: self.max_datagram,
            });
        }
        self.socket.send(dgram).await?;
        Ok(())
    }

    async fn recv(&self) -> Result<Vec<u8>, TransportError> {
        let mut buf = vec![0u8; self.max_datagram];
        let n = self.socket.recv(&mut buf).await?;
        buf.truncate(n);
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCAL: &str = "127.0.0.1:0";

    /// A connected pair of transports on localhost, `(a, b)`, each serving the
    /// other. Uses ephemeral ports so parallel tests never collide.
    async fn pair(conn: ConnId) -> (UdpTransport, UdpTransport) {
        let local: SocketAddr = LOCAL.parse().unwrap();
        // Bind both first so each knows the other's OS-assigned port.
        let a_sock = UdpSocket::bind(local).await.unwrap();
        let b_sock = UdpSocket::bind(local).await.unwrap();
        let a_addr = a_sock.local_addr().unwrap();
        let b_addr = b_sock.local_addr().unwrap();
        a_sock.connect(b_addr).await.unwrap();
        b_sock.connect(a_addr).await.unwrap();
        let a = UdpTransport {
            conn,
            socket: a_sock,
            max_datagram: DEFAULT_MAX_DATAGRAM,
        };
        let b = UdpTransport {
            conn,
            socket: b_sock,
            max_datagram: DEFAULT_MAX_DATAGRAM,
        };
        (a, b)
    }

    #[tokio::test]
    async fn connect_reports_bound_and_peer_addresses() {
        let (a, b) = pair(ConnId::new(1)).await;
        assert_eq!(a.conn_id(), ConnId::new(1));
        // a's peer is b's local addr, and vice-versa.
        assert_eq!(a.peer_addr().unwrap(), b.local_addr().unwrap());
        assert_eq!(b.peer_addr().unwrap(), a.local_addr().unwrap());
        // Port 0 was resolved to a real ephemeral port.
        assert_ne!(a.local_addr().unwrap().port(), 0);
    }

    #[tokio::test]
    async fn datagrams_move_both_ways() {
        let (a, b) = pair(ConnId::new(7)).await;
        a.send(b"ping").await.unwrap();
        assert_eq!(b.recv().await.unwrap(), b"ping");
        b.send(b"pong").await.unwrap();
        assert_eq!(a.recv().await.unwrap(), b"pong");
    }

    #[tokio::test]
    async fn empty_datagram_roundtrips() {
        let (a, b) = pair(ConnId::new(8)).await;
        a.send(b"").await.unwrap();
        assert_eq!(b.recv().await.unwrap(), b"");
    }

    #[tokio::test]
    async fn oversize_datagram_is_rejected_before_the_wire() {
        let (a, _b) = pair(ConnId::new(2)).await;
        let big = vec![0u8; DEFAULT_MAX_DATAGRAM + 1];
        match a.send(&big).await {
            Err(TransportError::TooLarge { size, max }) => {
                assert_eq!(size, DEFAULT_MAX_DATAGRAM + 1);
                assert_eq!(max, DEFAULT_MAX_DATAGRAM);
            }
            other => panic!("expected TooLarge, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn a_maximal_datagram_still_sends() {
        let (a, b) = pair(ConnId::new(3)).await;
        let payload = vec![0xABu8; DEFAULT_MAX_DATAGRAM];
        a.send(&payload).await.unwrap();
        assert_eq!(b.recv().await.unwrap(), payload);
    }
}
