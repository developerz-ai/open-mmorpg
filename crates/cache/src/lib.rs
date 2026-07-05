//! Dragonfly-backed ephemeral state: presence, positions-in-flight, chat fan-out,
//! rate-limit counters — anything that is safe to lose on restart.
//!
//! # The anti-dupe boundary
//!
//! This crate exposes **no ownership-write API**. There is deliberately no
//! function here to move an item between characters or mutate a balance, so the
//! dupe path (writing ownership through a cache that can drop data) *does not
//! compile*. Durable ownership goes through `omm-persistence` in a transaction —
//! that is the only crate allowed to write it. See
//! docs/architecture/04-data-and-consistency.md.

use omm_protocol::CharacterId;
use std::collections::HashMap;

/// Ephemeral key/value + presence store. The trait is what the shard depends on;
/// the Dragonfly client is one implementation, [`MemoryCache`] is another (tests).
pub trait EphemeralStore {
    /// Set an ephemeral string value. Loss on restart is acceptable by design.
    fn set(&mut self, key: &str, value: &str);
    /// Read an ephemeral value, if present.
    fn get(&self, key: &str) -> Option<String>;
    /// Mark a character present on this node.
    fn mark_present(&mut self, who: CharacterId);
    /// Whether a character is currently marked present.
    fn is_present(&self, who: CharacterId) -> bool;
}

/// In-memory reference implementation. Used in tests and single-process dev.
#[derive(Debug, Default)]
pub struct MemoryCache {
    kv: HashMap<String, String>,
    present: std::collections::HashSet<u64>,
}

impl MemoryCache {
    /// An empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl EphemeralStore for MemoryCache {
    fn set(&mut self, key: &str, value: &str) {
        self.kv.insert(key.to_owned(), value.to_owned());
    }

    fn get(&self, key: &str) -> Option<String> {
        self.kv.get(key).cloned()
    }

    fn mark_present(&mut self, who: CharacterId) {
        self.present.insert(who.raw());
    }

    fn is_present(&self, who: CharacterId) -> bool {
        self.present.contains(&who.raw())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kv_roundtrips() {
        let mut c = MemoryCache::new();
        c.set("zone:1:weather", "storm");
        assert_eq!(c.get("zone:1:weather").as_deref(), Some("storm"));
        assert_eq!(c.get("missing"), None);
    }

    #[test]
    fn presence_tracks_characters() {
        let mut c = MemoryCache::new();
        let who = CharacterId::new(7);
        assert!(!c.is_present(who));
        c.mark_present(who);
        assert!(c.is_present(who));
    }
}
