//! In-memory loopback [`Transport`]: a tokio `mpsc` pipe pair standing in for a
//! real UDP link in tests and single-process runs.
//!
//! Two [`Loopback`] endpoints share one connection: whatever `a` sends lands in
//! `b`'s inbox and vice-versa. The channel itself is reliable and ordered, so a
//! [`LossModel`] injects the *unreliability* — a caller-supplied predicate over
//! the datagram's **send ordinal** (0-based, per direction) decides which
//! datagrams vanish. Keying loss on the ordinal, not any frame sequence, means a
//! retransmit of the same frame gets an independent drop decision — exactly like
//! a real link, and the reason a reliability retransmit loop converges over it.
//!
//! Determinism is the whole point: no clocks, no RNG, no sockets. A CI run drives
//! the same loss pattern every time, so lossy-link behaviour is reproducible.

use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::{mpsc, Mutex};

use super::{framing, ConnId, Transport, TransportError};

/// Largest datagram the loopback will carry: one maximal [`framing::Frame`].
/// A real UDP transport enforces the path MTU instead; this cap just gives
/// [`TransportError::TooLarge`] a home so oversize sends fail the same way.
pub const MAX_DGRAM: usize = framing::OVERHEAD + framing::MAX_PAYLOAD;

/// Deterministic, injectable packet loss for one direction of a [`Loopback`].
///
/// The predicate is called with each datagram's send ordinal (the 0-based count
/// of datagrams offered to [`Transport::send`] on this endpoint); returning
/// `true` drops that datagram silently, as a lossy link would.
pub struct LossModel {
    drop: Box<dyn Fn(u64) -> bool + Send + Sync>,
}

impl LossModel {
    /// A perfect link: nothing is ever dropped.
    #[must_use]
    pub fn lossless() -> Self {
        Self {
            drop: Box::new(|_| false),
        }
    }

    /// A link whose loss is decided by `drop_seq_fn(ordinal)`. For CI-stable
    /// loss, keep it pure: e.g. `LossModel::new(|n| n % 10 < 3)` drops 30%.
    #[must_use]
    pub fn new(drop_seq_fn: impl Fn(u64) -> bool + Send + Sync + 'static) -> Self {
        Self {
            drop: Box::new(drop_seq_fn),
        }
    }

    fn drops(&self, ordinal: u64) -> bool {
        (self.drop)(ordinal)
    }
}

impl core::fmt::Debug for LossModel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("LossModel(..)")
    }
}

/// One endpoint of an in-memory connection. Holds the send half toward its peer
/// and the (mutex-guarded) receive half from it, plus this direction's loss
/// model and send counter. `Send + Sync`, so a sender and receiver task can
/// share one behind an `Arc<dyn Transport>` just like the UDP transport.
pub struct Loopback {
    conn: ConnId,
    tx: mpsc::UnboundedSender<Vec<u8>>,
    rx: Mutex<mpsc::UnboundedReceiver<Vec<u8>>>,
    loss: LossModel,
    sent: AtomicU64,
}

impl Loopback {
    /// A connected, perfectly-reliable pair sharing `conn`.
    #[must_use]
    pub fn pair(conn: ConnId) -> (Self, Self) {
        Self::pair_with_loss(conn, LossModel::lossless(), LossModel::lossless())
    }

    /// A connected pair where `a`'s outbound datagrams obey `a_loss` and `b`'s
    /// obey `b_loss` — model a lossy uplink with a clean downlink, or both.
    #[must_use]
    pub fn pair_with_loss(conn: ConnId, a_loss: LossModel, b_loss: LossModel) -> (Self, Self) {
        let (a_tx, b_rx) = mpsc::unbounded_channel();
        let (b_tx, a_rx) = mpsc::unbounded_channel();
        let a = Self {
            conn,
            tx: a_tx,
            rx: Mutex::new(a_rx),
            loss: a_loss,
            sent: AtomicU64::new(0),
        };
        let b = Self {
            conn,
            tx: b_tx,
            rx: Mutex::new(b_rx),
            loss: b_loss,
            sent: AtomicU64::new(0),
        };
        (a, b)
    }

    /// Non-blocking receive: the next queued datagram, or `None` if the inbox is
    /// momentarily empty. For a tick loop that drains all pending intents each
    /// frame and must never block on I/O. `Closed` once the peer is gone *and*
    /// the queue is drained.
    pub async fn try_recv(&self) -> Result<Option<Vec<u8>>, TransportError> {
        match self.rx.lock().await.try_recv() {
            Ok(dgram) => Ok(Some(dgram)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => Err(TransportError::Closed(self.conn)),
        }
    }
}

impl core::fmt::Debug for Loopback {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Loopback")
            .field("conn", &self.conn)
            .finish_non_exhaustive()
    }
}

#[async_trait::async_trait]
impl Transport for Loopback {
    fn conn_id(&self) -> ConnId {
        self.conn
    }

    async fn send(&self, dgram: &[u8]) -> Result<(), TransportError> {
        if dgram.len() > MAX_DGRAM {
            return Err(TransportError::TooLarge {
                size: dgram.len(),
                max: MAX_DGRAM,
            });
        }
        let ordinal = self.sent.fetch_add(1, Ordering::Relaxed);
        if self.loss.drops(ordinal) {
            return Ok(()); // best-effort: dropped, exactly as a UDP transport may.
        }
        self.tx
            .send(dgram.to_vec())
            .map_err(|_| TransportError::Closed(self.conn))
    }

    async fn recv(&self) -> Result<Vec<u8>, TransportError> {
        self.rx
            .lock()
            .await
            .recv()
            .await
            .ok_or(TransportError::Closed(self.conn))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{AckTracker, Frame};

    fn frame(seq: u16, payload: Vec<u8>) -> Vec<u8> {
        Frame::new(seq, 0, 0, payload).encode().unwrap()
    }

    #[tokio::test]
    async fn a_lossless_pair_moves_bytes_both_ways() {
        let (a, b) = Loopback::pair(ConnId::new(9));
        assert_eq!(a.conn_id(), ConnId::new(9));
        assert_eq!(b.conn_id(), ConnId::new(9));

        a.send(b"ping").await.unwrap();
        assert_eq!(b.recv().await.unwrap(), b"ping");
        b.send(b"pong").await.unwrap();
        assert_eq!(a.recv().await.unwrap(), b"pong");
    }

    #[tokio::test]
    async fn oversize_datagram_is_rejected() {
        let (a, _b) = Loopback::pair(ConnId::new(4));
        let big = vec![0u8; MAX_DGRAM + 1];
        match a.send(&big).await {
            Err(TransportError::TooLarge { size, max }) => {
                assert_eq!(size, MAX_DGRAM + 1);
                assert_eq!(max, MAX_DGRAM);
            }
            other => panic!("expected TooLarge, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn send_and_recv_report_closed_after_peer_drops() {
        let (a, b) = Loopback::pair(ConnId::new(5));
        drop(a);
        match b.recv().await {
            Err(TransportError::Closed(id)) => assert_eq!(id, ConnId::new(5)),
            other => panic!("expected Closed on recv, got {other:?}"),
        }

        let (c, d) = Loopback::pair(ConnId::new(6));
        drop(d);
        match c.send(b"x").await {
            Err(TransportError::Closed(id)) => assert_eq!(id, ConnId::new(6)),
            other => panic!("expected Closed on send, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn reliable_delivery_under_thirty_percent_loss() {
        // 30% deterministic loss on the uplink; the downlink stays clean.
        let loss = LossModel::new(|n| n % 10 < 3);
        let (client, server) =
            Loopback::pair_with_loss(ConnId::new(1), loss, LossModel::lossless());

        let total: u16 = 64;
        let mut delivered: BTreeSet<u16> = BTreeSet::new();
        let mut acks = AckTracker::new();
        let mut unacked: BTreeSet<u16> = (0..total).collect();

        let mut round = 0;
        while !unacked.is_empty() {
            round += 1;
            assert!(
                round <= 50,
                "retransmit must converge; stuck on {unacked:?}"
            );
            for &seq in &unacked {
                client
                    .send(&frame(seq, seq.to_le_bytes().to_vec()))
                    .await
                    .unwrap();
            }
            while let Some(buf) = server.try_recv().await.unwrap() {
                let (f, _) = Frame::decode(&buf).unwrap();
                delivered.insert(f.seq);
                acks.record(f.seq);
            }
            unacked.retain(|seq| !delivered.contains(seq));
        }

        // Every payload arrived despite loss, and the ack window reached the top.
        assert_eq!(delivered.len(), usize::from(total));
        assert_eq!(acks.latest(), total - 1);
    }

    #[tokio::test]
    async fn duplicates_and_reorder_are_tolerated() {
        let (client, server) = Loopback::pair(ConnId::new(2));
        // Out of order, with seq 5 duplicated — the transport delivers verbatim.
        for &seq in &[5u16, 3, 5, 4, 2, 1] {
            client.send(&frame(seq, vec![seq as u8])).await.unwrap();
        }
        drop(client); // let the receiver terminate once the queue is drained.

        let mut acks = AckTracker::new();
        let mut count = 0;
        loop {
            match server.recv().await {
                Ok(buf) => {
                    let (f, _) = Frame::decode(&buf).unwrap();
                    acks.record(f.seq);
                    count += 1;
                }
                Err(TransportError::Closed(_)) => break,
                Err(e) => panic!("unexpected {e:?}"),
            }
        }
        assert_eq!(count, 6, "loopback delivers every datagram, dup included");
        // Reorder + dup leave the reliability view correct: 1..=5 all acked.
        for seq in 1..=5 {
            assert!(acks.is_acked(seq), "seq {seq} should be acked");
        }
        assert_eq!(acks.latest(), 5);
    }

    #[tokio::test]
    async fn ack_window_advances_as_frames_arrive() {
        let (client, server) = Loopback::pair(ConnId::new(3));
        let mut acks = AckTracker::new();
        for seq in 1..=5u16 {
            client.send(&frame(seq, Vec::new())).await.unwrap();
            let buf = server.recv().await.unwrap();
            let (f, _) = Frame::decode(&buf).unwrap();
            acks.record(f.seq);
            assert_eq!(acks.latest(), seq, "latest advances with each arrival");
        }
        // latest = 5; the four before it (4,3,2,1) sit in the low nibble.
        assert_eq!(acks.ack_bits() & 0b1111, 0b1111);
    }
}
