//! The **world feed**: a GTA-inspired stream of notable world events (a boss
//! falls, a rare drops, a keep is taken). Shards publish events across the bus;
//! worldsvc aggregates them into a bounded, newest-first feed the web reads.
//!
//! It is a projection, not state-of-record — losing it costs nothing durable, so
//! it lives in memory (backed by Dragonfly fan-out in production).

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// One notable world event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedEntry {
    /// Category tag (e.g. `"boss"`, `"pvp"`, `"system"`) — the client styles by it.
    pub kind: String,
    /// Human-facing message. The web localizes via its own i18n; this is the
    /// event payload, not a UI string.
    pub message: String,
    /// The sim tick the event occurred on, for ordering across shards.
    pub tick: u64,
}

/// A bounded ring of the most recent world events.
#[derive(Debug)]
pub struct WorldFeed {
    entries: VecDeque<FeedEntry>,
    cap: usize,
}

impl WorldFeed {
    /// A feed that retains at most `cap` entries (oldest dropped past that).
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            cap: cap.max(1),
        }
    }

    /// Publish an event, evicting the oldest if at capacity.
    pub fn publish(&mut self, entry: FeedEntry) {
        if self.entries.len() == self.cap {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// The most recent events, newest first, capped at `limit`.
    #[must_use]
    pub fn recent(&self, limit: usize) -> Vec<FeedEntry> {
        self.entries.iter().rev().take(limit).cloned().collect()
    }
}

impl Default for WorldFeed {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(msg: &str, tick: u64) -> FeedEntry {
        FeedEntry {
            kind: "system".into(),
            message: msg.into(),
            tick,
        }
    }

    #[test]
    fn recent_is_newest_first_and_limited() {
        let mut f = WorldFeed::new(10);
        f.publish(entry("a", 1));
        f.publish(entry("b", 2));
        f.publish(entry("c", 3));
        let r = f.recent(2);
        assert_eq!(
            r.iter().map(|e| e.message.as_str()).collect::<Vec<_>>(),
            ["c", "b"]
        );
    }

    #[test]
    fn ring_evicts_oldest_at_capacity() {
        let mut f = WorldFeed::new(2);
        f.publish(entry("a", 1));
        f.publish(entry("b", 2));
        f.publish(entry("c", 3));
        let recent = f.recent(5);
        assert_eq!(recent.len(), 2);
        assert_eq!(
            recent.iter().map(|e| e.message.clone()).collect::<Vec<_>>(),
            ["c", "b"]
        );
    }
}
