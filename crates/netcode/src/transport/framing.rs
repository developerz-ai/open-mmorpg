//! Length-prefixed reliability framing over a datagram [`super::Transport`].
//!
//! Every frame carries the header a receiver needs to drive its
//! [`crate::reliability::AckTracker`]: a sending sequence number, the latest
//! sequence the sender has seen from the peer, and the 32-bit bitfield of the 32
//! sequences before it. On the wire (all integers little-endian):
//!
//! ```text
//! | len: u16 | seq: u16 | ack: u16 | ack_bits: u32 | payload: [u8; len] |
//! ```
//!
//! `len` counts only the payload; the 8-byte header is fixed. Length-prefixing
//! lets several frames share one datagram (or stream) unambiguously — [`Frame::decode`]
//! reports how many bytes it consumed so a caller can parse the next one.

use crate::reliability::AckTracker;

/// Fixed header after the length prefix: seq(2) + ack(2) + ack_bits(4).
pub const HEADER_LEN: usize = 8;
/// The length prefix itself: a `u16` payload length.
pub const LEN_PREFIX: usize = 2;
/// Total per-frame overhead on the wire (prefix + header).
pub const OVERHEAD: usize = LEN_PREFIX + HEADER_LEN;
/// Largest payload one frame can carry, bounded by the `u16` length prefix.
pub const MAX_PAYLOAD: usize = u16::MAX as usize;

/// A single reliability frame: the ack header plus an opaque payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// This frame's own sequence number (wraps at `u16::MAX`).
    pub seq: u16,
    /// The latest sequence the sender has received from the peer.
    pub ack: u16,
    /// Bitfield of the 32 sequences before `ack` that were received.
    pub ack_bits: u32,
    /// The opaque payload (a serialized snapshot delta, input, etc.).
    pub payload: Vec<u8>,
}

/// A frame that could not be parsed or serialized.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FramingError {
    /// The buffer ended before a full frame was present.
    #[error("truncated frame: need {need} bytes, got {got}")]
    Truncated {
        /// Bytes the frame requires.
        need: usize,
        /// Bytes actually available.
        got: usize,
    },
    /// The payload is larger than a `u16` length prefix can describe.
    #[error("payload too large: {0} bytes (max {max})", max = MAX_PAYLOAD)]
    PayloadTooLarge(usize),
}

impl Frame {
    /// A frame with an explicit ack header.
    #[must_use]
    pub const fn new(seq: u16, ack: u16, ack_bits: u32, payload: Vec<u8>) -> Self {
        Self {
            seq,
            ack,
            ack_bits,
            payload,
        }
    }

    /// A frame stamping the current ack state from `acks` — the tracker of what
    /// this side has received from the peer. Copies [`AckTracker::latest`] into
    /// `ack` and [`AckTracker::ack_bits`] into `ack_bits`.
    #[must_use]
    pub fn with_acks(seq: u16, acks: &AckTracker, payload: Vec<u8>) -> Self {
        Self::new(seq, acks.latest(), acks.ack_bits(), payload)
    }

    /// Serialize to bytes. Errors only if the payload exceeds [`MAX_PAYLOAD`].
    pub fn encode(&self) -> Result<Vec<u8>, FramingError> {
        let len = u16::try_from(self.payload.len())
            .map_err(|_| FramingError::PayloadTooLarge(self.payload.len()))?;
        let mut buf = Vec::with_capacity(OVERHEAD + self.payload.len());
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&self.seq.to_le_bytes());
        buf.extend_from_slice(&self.ack.to_le_bytes());
        buf.extend_from_slice(&self.ack_bits.to_le_bytes());
        buf.extend_from_slice(&self.payload);
        Ok(buf)
    }

    /// Parse one frame from the front of `buf`, returning it and the number of
    /// bytes consumed (so a caller can parse the next frame from `buf[n..]`).
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), FramingError> {
        if buf.len() < OVERHEAD {
            return Err(FramingError::Truncated {
                need: OVERHEAD,
                got: buf.len(),
            });
        }
        let payload_len = usize::from(u16::from_le_bytes([buf[0], buf[1]]));
        let total = OVERHEAD + payload_len;
        if buf.len() < total {
            return Err(FramingError::Truncated {
                need: total,
                got: buf.len(),
            });
        }
        let seq = u16::from_le_bytes([buf[2], buf[3]]);
        let ack = u16::from_le_bytes([buf[4], buf[5]]);
        let ack_bits = u32::from_le_bytes([buf[6], buf[7], buf[8], buf[9]]);
        let payload = buf[OVERHEAD..total].to_vec();
        Ok((Self::new(seq, ack, ack_bits, payload), total))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reliability::seq_greater_than;
    use proptest::prelude::*;

    #[test]
    fn roundtrip_preserves_header_and_payload() {
        let f = Frame::new(7, 42, 0xDEAD_BEEF, b"hello".to_vec());
        let bytes = f.encode().unwrap();
        assert_eq!(bytes.len(), OVERHEAD + 5);
        let (got, n) = Frame::decode(&bytes).unwrap();
        assert_eq!(got, f);
        assert_eq!(n, bytes.len());
    }

    #[test]
    fn frame_carries_ack_tracker_state() {
        let mut acks = AckTracker::new();
        acks.record(41);
        acks.record(42);
        let f = Frame::with_acks(7, &acks, b"hi".to_vec());
        assert_eq!(f.ack, 42);
        assert_eq!(f.ack_bits, acks.ack_bits());

        // Round-trip, then let the receiver drive its own tracker off the frame.
        let bytes = f.encode().unwrap();
        let (got, _) = Frame::decode(&bytes).unwrap();
        assert_eq!(got.seq, 7);
        assert!(seq_greater_than(got.ack, 41)); // 42 is newer than 41
    }

    #[test]
    fn decode_reports_bytes_consumed_for_packed_frames() {
        let a = Frame::new(1, 0, 0, b"aa".to_vec());
        let b = Frame::new(2, 1, 1, b"bbbb".to_vec());
        let mut buf = a.encode().unwrap();
        buf.extend(b.encode().unwrap());

        let (first, n) = Frame::decode(&buf).unwrap();
        assert_eq!(first, a);
        let (second, m) = Frame::decode(&buf[n..]).unwrap();
        assert_eq!(second, b);
        assert_eq!(n + m, buf.len());
    }

    #[test]
    fn short_buffer_is_truncated_not_a_panic() {
        assert_eq!(
            Frame::decode(&[0u8; 4]),
            Err(FramingError::Truncated {
                need: OVERHEAD,
                got: 4
            })
        );
        // Header claims 8 payload bytes but only 2 follow.
        let mut buf = 8u16.to_le_bytes().to_vec();
        buf.extend_from_slice(&[0u8; HEADER_LEN]);
        buf.extend_from_slice(&[1, 2]);
        assert_eq!(
            Frame::decode(&buf),
            Err(FramingError::Truncated {
                need: OVERHEAD + 8,
                got: buf.len(),
            })
        );
    }

    proptest! {
        #[test]
        fn encode_decode_is_identity(
            seq in any::<u16>(),
            ack in any::<u16>(),
            ack_bits in any::<u32>(),
            payload in proptest::collection::vec(any::<u8>(), 0..2048),
        ) {
            let f = Frame::new(seq, ack, ack_bits, payload);
            let bytes = f.encode().unwrap();
            let (got, n) = Frame::decode(&bytes).unwrap();
            prop_assert_eq!(&got, &f);
            prop_assert_eq!(n, OVERHEAD + f.payload.len());
        }
    }
}
