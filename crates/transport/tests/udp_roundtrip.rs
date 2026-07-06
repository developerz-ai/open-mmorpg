//! Smoke test: one framed UDP roundtrip over real loopback sockets.
//!
//! Both halves bind `127.0.0.1:0` (OS-assigned ephemeral ports) so parallel
//! nextest workers never collide. The test encodes a [`Frame`] at the sender,
//! ships it through the socket, and decodes it on the other side — exercising
//! the full stack from framing down to the wire without any mocking.
//!
//! Headless-safe: only loopback, no multicast or broadcast, no fixed port.

use omm_netcode::transport::{ConnId, Frame, Transport};
use omm_transport::{UdpTransport, DEFAULT_MAX_DATAGRAM};
use tokio::net::UdpSocket;

/// Bind two independent sockets, cross-connect them, and wrap both as
/// [`UdpTransport`] endpoints. Returns `(client, server)`.
#[allow(clippy::unwrap_used)] // test helper; all operations are on loopback sockets
async fn loopback_pair(conn: ConnId) -> (UdpTransport, UdpTransport) {
    let local: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let a_sock = UdpSocket::bind(local).await.unwrap();
    let b_sock = UdpSocket::bind(local).await.unwrap();
    let a_addr = a_sock.local_addr().unwrap();
    let b_addr = b_sock.local_addr().unwrap();
    a_sock.connect(b_addr).await.unwrap();
    b_sock.connect(a_addr).await.unwrap();
    let client = UdpTransport::from_connected(conn, a_sock, DEFAULT_MAX_DATAGRAM);
    let server = UdpTransport::from_connected(conn, b_sock, DEFAULT_MAX_DATAGRAM);
    (client, server)
}

/// One framed message in each direction, verifying payload integrity and that
/// the framing header survives the wire intact.
#[tokio::test]
async fn framed_roundtrip() {
    let conn = ConnId::new(1);
    let (client, server) = loopback_pair(conn).await;

    // client → server
    let ping_payload = b"hello server".to_vec();
    let ping_frame = Frame::new(1, 0, 0, ping_payload.clone());
    let encoded = ping_frame
        .encode()
        .expect("encode must not fail for small payload");
    client.send(&encoded).await.expect("client send");

    let raw = server.recv().await.expect("server recv");
    let (decoded, consumed) = Frame::decode(&raw).expect("server decode");
    assert_eq!(
        consumed,
        raw.len(),
        "frame must consume all bytes in one datagram"
    );
    assert_eq!(decoded.seq, 1);
    assert_eq!(decoded.payload, ping_payload);

    // server → client (echo with incremented seq and ack)
    let pong_payload = b"hello client".to_vec();
    let pong_frame = Frame::new(1, 1, 0b1, pong_payload.clone());
    let encoded = pong_frame.encode().expect("encode");
    server.send(&encoded).await.expect("server send");

    let raw = client.recv().await.expect("client recv");
    let (decoded, _) = Frame::decode(&raw).expect("client decode");
    assert_eq!(decoded.seq, 1);
    assert_eq!(decoded.ack, 1, "server echoed ack back");
    assert_eq!(decoded.payload, pong_payload);
}

/// conn_id survives construction and is equal on both halves (one logical
/// connection shared by two socket endpoints).
#[tokio::test]
async fn conn_id_is_correct() {
    let (client, server) = loopback_pair(ConnId::new(42)).await;
    assert_eq!(client.conn_id(), ConnId::new(42));
    assert_eq!(server.conn_id(), ConnId::new(42));
}

/// OS resolves port 0 to a real ephemeral port; both addresses must be distinct.
#[tokio::test]
async fn ephemeral_ports_are_assigned() {
    let (client, server) = loopback_pair(ConnId::new(2)).await;
    let c_addr = client.local_addr().unwrap();
    let s_addr = server.local_addr().unwrap();
    assert_ne!(c_addr.port(), 0, "client port must not stay 0");
    assert_ne!(s_addr.port(), 0, "server port must not stay 0");
    assert_ne!(c_addr, s_addr, "endpoints must be on distinct ports");
    // Each is correctly cross-wired.
    assert_eq!(client.peer_addr().unwrap(), s_addr);
    assert_eq!(server.peer_addr().unwrap(), c_addr);
}

/// An oversize datagram is rejected before hitting the wire — the frame is
/// never delivered to the peer (no recv() call on the other side needed).
#[tokio::test]
async fn oversize_datagram_rejected() {
    use omm_netcode::transport::TransportError;
    let (client, _server) = loopback_pair(ConnId::new(3)).await;
    let big = vec![0xFFu8; DEFAULT_MAX_DATAGRAM + 1];
    match client.send(&big).await {
        Err(TransportError::TooLarge { size, max }) => {
            assert_eq!(size, DEFAULT_MAX_DATAGRAM + 1);
            assert_eq!(max, DEFAULT_MAX_DATAGRAM);
        }
        other => panic!("expected TooLarge, got {other:?}"),
    }
}
