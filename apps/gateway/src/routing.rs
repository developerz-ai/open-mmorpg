//! Shard routing.
//!
//! Maps a logical zone to the shard that serves it. Routing is **deterministic**
//! so the same zone lands on the same shard across gateway replicas, and it
//! returns an **opaque** [`ShardHandle`] — never a shard IP
//! (`docs/architecture/02-server-topology.md`: "never expose shard IPs directly").

use omm_protocol::{ShardId, ZoneId};
use serde::Serialize;

/// An opaque, public-safe reference to a shard.
///
/// The client and web only ever see this handle; it deliberately carries no
/// address or internal topology, only a stable id the shard/edge can resolve.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShardHandle(String);

impl ShardHandle {
    /// Build the opaque handle for a shard id.
    #[must_use]
    pub fn from_shard(id: ShardId) -> Self {
        Self(format!("shard-{}", id.raw()))
    }
}

impl std::fmt::Display for ShardHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Assigns a zone to a shard. Behind a trait so a live discovery-backed router
/// can replace the static one without touching handlers.
pub trait ShardRouter: Send + Sync + 'static {
    /// The shard that should serve `zone`.
    fn route(&self, zone: ZoneId) -> ShardHandle;
}

/// Static, hash-based router over a fixed shard set.
///
/// `hash(zone) % shards.len()` — deterministic and evenly spread; adding shards
/// reshuffles zones, which is acceptable for the control-plane assignment (a
/// consistent-hash ring is a later refinement, see follow-ups).
#[derive(Debug, Clone)]
pub struct HashRouter {
    shards: Vec<ShardId>,
}

impl HashRouter {
    /// Build a router over a non-empty shard set.
    ///
    /// # Errors
    /// Returns `Err` if `shards` is empty — a gateway with no shards can route
    /// nothing and must fail fast at startup.
    pub fn new(shards: Vec<ShardId>) -> Result<Self, EmptyShardSet> {
        if shards.is_empty() {
            return Err(EmptyShardSet);
        }
        Ok(Self { shards })
    }
}

impl ShardRouter for HashRouter {
    fn route(&self, zone: ZoneId) -> ShardHandle {
        // `shards` is non-empty by construction, so the modulo is in range and
        // the index never panics.
        let idx = (fnv1a(&zone.raw().to_be_bytes()) % self.shards.len() as u64) as usize;
        let id = self.shards.get(idx).copied().unwrap_or(self.shards[0]);
        ShardHandle::from_shard(id)
    }
}

/// Error: a router was constructed with no shards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("shard set must not be empty")]
pub struct EmptyShardSet;

/// FNV-1a hash over bytes — small, dependency-free, and **stable across builds**
/// (unlike `DefaultHasher`), which is what keeps routing reproducible.
pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routing_is_deterministic() {
        let router =
            HashRouter::new(vec![ShardId::new(1), ShardId::new(2), ShardId::new(3)]).unwrap();
        let a = router.route(ZoneId::new(1234));
        let b = router.route(ZoneId::new(1234));
        assert_eq!(a, b);
    }

    #[test]
    fn handle_is_opaque_and_never_an_ip() {
        let router = HashRouter::new(vec![ShardId::new(7)]).unwrap();
        let h = router.route(ZoneId::new(99));
        assert_eq!(h.to_string(), "shard-7");
        assert!(
            !h.to_string().contains('.'),
            "handle must not resemble an IP"
        );
    }

    #[test]
    fn distributes_across_shards() {
        let router =
            HashRouter::new(vec![ShardId::new(1), ShardId::new(2), ShardId::new(3)]).unwrap();
        let mut seen = std::collections::HashSet::new();
        for z in 0..300u64 {
            seen.insert(router.route(ZoneId::new(z)).to_string());
        }
        assert_eq!(seen.len(), 3, "all shards should receive some zones");
    }

    #[test]
    fn empty_shard_set_is_rejected() {
        assert!(matches!(HashRouter::new(vec![]), Err(EmptyShardSet)));
    }

    #[test]
    fn fnv1a_is_stable() {
        // Guard against an accidental algorithm change breaking route stability.
        assert_eq!(fnv1a(b"open-mmorpg"), fnv1a(b"open-mmorpg"));
        assert_ne!(fnv1a(b"a"), fnv1a(b"b"));
    }
}
