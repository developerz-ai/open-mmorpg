//! The social plane: cross-shard **chat** channels and **guild** membership.
//! Both span shards, so they live here rather than on any one shard. Chat is
//! ephemeral (bounded per-channel history); guild rosters are a read projection
//! of durable state (the authoritative guild membership write goes through
//! `persistence` — this serves the reads the web consumes).

use omm_errors::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// A single chat line on a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Sender character id (raw).
    pub sender: u64,
    /// Message body.
    pub body: String,
    /// Sim tick sent on, for ordering.
    pub tick: u64,
}

/// Bounded per-channel chat history, fanned out across shards.
#[derive(Debug, Default)]
pub struct ChatHub {
    channels: BTreeMap<String, VecDeque<ChatMessage>>,
    cap: usize,
}

impl ChatHub {
    /// A hub keeping at most `cap` recent messages per channel.
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            channels: BTreeMap::new(),
            cap: cap.max(1),
        }
    }

    /// Post a message to `channel`, evicting the oldest past capacity.
    pub fn post(&mut self, channel: &str, msg: ChatMessage) {
        let log = self.channels.entry(channel.to_owned()).or_default();
        if log.len() == self.cap {
            log.pop_front();
        }
        log.push_back(msg);
    }

    /// Recent messages on `channel`, newest first, capped at `limit`.
    #[must_use]
    pub fn recent(&self, channel: &str, limit: usize) -> Vec<ChatMessage> {
        self.channels
            .get(channel)
            .map(|log| log.iter().rev().take(limit).cloned().collect())
            .unwrap_or_default()
    }
}

/// A guild id (worldsvc-issued for the projection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GuildId(pub u64);

/// A guild and its membership roster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Guild {
    pub id: GuildId,
    pub name: String,
    /// Member character ids (raw), ordered for deterministic rosters.
    pub members: BTreeSet<u64>,
}

/// Registry of guilds — the cross-shard membership projection.
#[derive(Debug, Default)]
pub struct GuildRegistry {
    guilds: BTreeMap<GuildId, Guild>,
    next_id: u64,
}

impl GuildRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a guild with `founder` as its first member. Returns its id.
    ///
    /// # Errors
    /// [`CoreError::BadRequest`] if `name` is blank.
    pub fn create(&mut self, name: &str, founder: u64) -> CoreResult<GuildId> {
        if name.trim().is_empty() {
            return Err(CoreError::BadRequest("guild name is empty".into()));
        }
        self.next_id += 1;
        let id = GuildId(self.next_id);
        let mut members = BTreeSet::new();
        members.insert(founder);
        self.guilds.insert(
            id,
            Guild {
                id,
                name: name.to_owned(),
                members,
            },
        );
        Ok(id)
    }

    /// Add `member` to a guild.
    ///
    /// # Errors
    /// [`CoreError::NotFound`] if the guild does not exist.
    pub fn join(&mut self, id: GuildId, member: u64) -> CoreResult<()> {
        let guild = self
            .guilds
            .get_mut(&id)
            .ok_or_else(|| CoreError::NotFound(format!("guild {}", id.0)))?;
        guild.members.insert(member);
        Ok(())
    }

    /// Fetch a guild (roster included).
    #[must_use]
    pub fn get(&self, id: GuildId) -> Option<&Guild> {
        self.guilds.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_keeps_recent_newest_first() {
        let mut hub = ChatHub::new(8);
        for t in 1..=3 {
            hub.post(
                "trade",
                ChatMessage {
                    sender: 1,
                    body: format!("m{t}"),
                    tick: t,
                },
            );
        }
        let r = hub.recent("trade", 2);
        assert_eq!(
            r.iter().map(|m| m.body.clone()).collect::<Vec<_>>(),
            ["m3", "m2"]
        );
        assert!(hub.recent("empty", 5).is_empty());
    }

    #[test]
    fn chat_evicts_oldest_per_channel() {
        let mut hub = ChatHub::new(2);
        for t in 1..=3 {
            hub.post(
                "g",
                ChatMessage {
                    sender: 1,
                    body: t.to_string(),
                    tick: t,
                },
            );
        }
        assert_eq!(hub.recent("g", 9).len(), 2);
    }

    #[test]
    fn guild_create_join_and_roster() {
        let mut reg = GuildRegistry::new();
        let id = reg.create("Dawnward", 10).unwrap();
        reg.join(id, 20).unwrap();
        reg.join(id, 20).unwrap(); // idempotent membership
        let g = reg.get(id).unwrap();
        assert_eq!(g.members.iter().copied().collect::<Vec<_>>(), [10, 20]);
    }

    #[test]
    fn guild_rejects_blank_name_and_unknown_join() {
        let mut reg = GuildRegistry::new();
        assert_eq!(
            reg.create("  ", 1).unwrap_err().code(),
            omm_errors::ClientCode::BadRequest
        );
        assert_eq!(
            reg.join(GuildId(99), 1).unwrap_err().code(),
            omm_errors::ClientCode::NotFound
        );
    }
}
