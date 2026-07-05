//! Edge rate limiting.
//!
//! A token-bucket limiter to blunt login floods and packet spam
//! (`docs/specs/game-server/security/README.md`: "Login floods / packet spam →
//! L7 rate-limit ... at the gateway"). The bucket maths are pure and clock is
//! injected, so they are unit-testable; the async [`KeyedRateLimiter`] wires a
//! monotonic clock and per-key state for the request path.

use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::Mutex;

/// A single token bucket. Clock is injected (`now_ms`) so refill/allow logic is
/// deterministic and testable without sleeping.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_per_ms: f64,
    last_ms: u64,
}

impl TokenBucket {
    /// A bucket that starts full, holding `capacity` tokens and refilling at
    /// `refill_per_sec` tokens/second.
    #[must_use]
    pub fn new(capacity: u32, refill_per_sec: f64, now_ms: u64) -> Self {
        Self {
            capacity: f64::from(capacity),
            tokens: f64::from(capacity),
            refill_per_ms: refill_per_sec / 1000.0,
            last_ms: now_ms,
        }
    }

    /// Try to spend one token at `now_ms`. Refills for elapsed time first, caps
    /// at capacity, then consumes a token if one is available.
    pub fn try_acquire(&mut self, now_ms: u64) -> bool {
        let elapsed = now_ms.saturating_sub(self.last_ms) as f64;
        self.tokens = (self.tokens + elapsed * self.refill_per_ms).min(self.capacity);
        self.last_ms = now_ms;
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Configuration for a [`KeyedRateLimiter`].
#[derive(Debug, Clone, Copy)]
pub struct RateConfig {
    /// Burst size — how many requests are allowed back-to-back.
    pub capacity: u32,
    /// Sustained rate, tokens refilled per second.
    pub refill_per_sec: f64,
}

impl Default for RateConfig {
    fn default() -> Self {
        // Conservative login default: 5-request burst, ~1/sec sustained.
        Self {
            capacity: 5,
            refill_per_sec: 1.0,
        }
    }
}

/// A rate limiter with one [`TokenBucket`] per key (IP or account), sharing one
/// [`RateConfig`]. Uses a `tokio::sync::Mutex` — never a blocking lock — so it
/// is safe on the async request path.
#[derive(Debug)]
pub struct KeyedRateLimiter {
    config: RateConfig,
    start: Instant,
    buckets: Mutex<HashMap<String, TokenBucket>>,
}

impl KeyedRateLimiter {
    /// Build a limiter from a config.
    #[must_use]
    pub fn new(config: RateConfig) -> Self {
        Self {
            config,
            start: Instant::now(),
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Monotonic milliseconds since construction — the runtime clock for
    /// buckets. Monotonic, so it cannot be skewed by wall-clock changes.
    fn now_ms(&self) -> u64 {
        // `as u64` saturates for practical uptimes (u64 ms ≈ 584M years).
        self.start.elapsed().as_millis() as u64
    }

    /// Record a request for `key`; returns `true` if allowed, `false` if the
    /// caller has exhausted its bucket.
    pub async fn check(&self, key: &str) -> bool {
        let now = self.now_ms();
        let mut buckets = self.buckets.lock().await;
        let bucket = buckets.entry(key.to_owned()).or_insert_with(|| {
            TokenBucket::new(self.config.capacity, self.config.refill_per_sec, now)
        });
        bucket.try_acquire(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_burst_then_blocks() {
        let mut b = TokenBucket::new(3, 1.0, 0);
        assert!(b.try_acquire(0));
        assert!(b.try_acquire(0));
        assert!(b.try_acquire(0));
        assert!(
            !b.try_acquire(0),
            "4th request in the same instant is blocked"
        );
    }

    #[test]
    fn refills_over_time() {
        let mut b = TokenBucket::new(2, 1.0, 0);
        assert!(b.try_acquire(0));
        assert!(b.try_acquire(0));
        assert!(!b.try_acquire(0));
        // One token refills after 1000ms at 1/sec.
        assert!(b.try_acquire(1_000));
        assert!(!b.try_acquire(1_000));
    }

    #[test]
    fn refill_caps_at_capacity() {
        let mut b = TokenBucket::new(2, 1.0, 0);
        assert!(b.try_acquire(0));
        assert!(b.try_acquire(0));
        // Idle for an hour — must not exceed capacity of 2.
        assert!(b.try_acquire(3_600_000));
        assert!(b.try_acquire(3_600_000));
        assert!(!b.try_acquire(3_600_000));
    }

    #[tokio::test]
    async fn keyed_limiter_isolates_keys() {
        let limiter = KeyedRateLimiter::new(RateConfig {
            capacity: 1,
            refill_per_sec: 0.0,
        });
        assert!(limiter.check("alice").await);
        assert!(!limiter.check("alice").await, "alice exhausted");
        assert!(limiter.check("bob").await, "bob has an independent bucket");
    }
}
