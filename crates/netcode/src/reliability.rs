//! Reliability primitives for the UDP transport.
//!
//! UDP gives us speed but no ordering or delivery guarantees, so we build a thin
//! reliability layer on top: wrapping 16-bit sequence numbers and a 32-entry ack
//! bitfield. This module is **pure math** — no sockets — so every edge case
//! (wrap-around, gaps, duplicates) is unit-tested deterministically. The socket
//! wiring lives in `apps/shard` / `apps/gateway`.

/// Half the sequence space; the threshold for "newer than" comparisons.
const HALF: u16 = 1 << 15;

/// RFC 1982-style comparison: is `a` newer than `b` in a wrapping sequence?
///
/// Handles the wrap boundary correctly: `1` is newer than `65535`.
#[must_use]
pub fn seq_greater_than(a: u16, b: u16) -> bool {
    ((a > b) && (a - b <= HALF)) || ((b > a) && (b - a > HALF))
}

/// Tracks which recent packets have been received, for building acks.
///
/// `latest` is the highest sequence seen; `bits` records the 32 sequences before
/// it (bit `n` set == `latest - 1 - n` was received).
#[derive(Debug, Clone, Copy, Default)]
pub struct AckTracker {
    latest: u16,
    bits: u32,
    seen_any: bool,
}

impl AckTracker {
    /// A fresh tracker that has seen nothing.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            latest: 0,
            bits: 0,
            seen_any: false,
        }
    }

    /// Record a received sequence number, updating the ack window.
    pub fn record(&mut self, seq: u16) {
        if !self.seen_any {
            self.latest = seq;
            self.seen_any = true;
            return;
        }
        if seq_greater_than(seq, self.latest) {
            let shift = seq.wrapping_sub(self.latest);
            // Old latest becomes a set bit; shift the window forward.
            self.bits = self.bits.checked_shl(u32::from(shift)).unwrap_or(0)
                | 1u32.checked_shl(u32::from(shift) - 1).unwrap_or(0);
            self.latest = seq;
        } else {
            let diff = self.latest.wrapping_sub(seq);
            if (1..=32).contains(&diff) {
                self.bits |= 1 << (diff - 1);
            }
        }
    }

    /// The highest sequence acknowledged so far.
    #[must_use]
    pub const fn latest(&self) -> u16 {
        self.latest
    }

    /// The 32-bit ack bitfield: bit `n` set == `latest() - 1 - n` was received.
    /// The framing layer stamps this onto every outgoing [`crate::Frame`] so the
    /// peer learns which recent packets arrived.
    #[must_use]
    pub const fn ack_bits(&self) -> u32 {
        self.bits
    }

    /// Whether `seq` is within the window and has been received.
    #[must_use]
    pub fn is_acked(&self, seq: u16) -> bool {
        if !self.seen_any {
            return false;
        }
        if seq == self.latest {
            return true;
        }
        let diff = self.latest.wrapping_sub(seq);
        (1..=32).contains(&diff) && (self.bits & (1 << (diff - 1))) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_across_wrap_boundary() {
        assert!(seq_greater_than(1, 65535));
        assert!(!seq_greater_than(65535, 1));
        assert!(seq_greater_than(100, 99));
        assert!(!seq_greater_than(99, 100));
    }

    #[test]
    fn tracker_acks_in_order() {
        let mut t = AckTracker::new();
        t.record(1);
        t.record(2);
        t.record(3);
        assert_eq!(t.latest(), 3);
        assert!(t.is_acked(3));
        assert!(t.is_acked(2));
        assert!(t.is_acked(1));
        assert!(!t.is_acked(4));
    }

    #[test]
    fn tracker_handles_gaps_and_late_arrivals() {
        let mut t = AckTracker::new();
        t.record(1);
        t.record(3); // 2 missing
        assert!(t.is_acked(3));
        assert!(t.is_acked(1));
        assert!(!t.is_acked(2));
        t.record(2); // arrives late
        assert!(t.is_acked(2));
    }

    #[test]
    fn ack_bits_expose_received_window() {
        let mut t = AckTracker::new();
        t.record(10);
        t.record(11); // 11 is latest; bit 0 == (latest - 1) == 10 received.
        assert_eq!(t.ack_bits() & 1, 1);
        t.record(13); // gap at 12; window shifts, 11 now two behind.
        assert_eq!(t.latest(), 13);
        assert_eq!(t.ack_bits() & 0b10, 0b10); // bit 1 == 11 received
        assert_eq!(t.ack_bits() & 0b1, 0); // bit 0 == 12 missing
    }
}
