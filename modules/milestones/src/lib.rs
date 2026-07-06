//! `milestones` — a compiled module that turns real server events into player
//! achievements, end to end and lock-free.
//!
//! It observes several core hooks — [`ServerHooks::on_player_login`],
//! [`ServerHooks::on_level_up`], [`ServerHooks::on_zone_enter`],
//! [`ServerHooks::on_chat`], [`ServerHooks::on_item_use`], and
//! [`ServerHooks::on_tick`] — accumulates monotonic counters in atomics (no
//! `Mutex`, so a hook never blocks the authoritative tick path), and unlocks a
//! [`Milestone`] the moment its threshold is crossed, logging each newly-earned
//! one exactly once.
//!
//! The *rules* (thresholds, bit layout) live in the pure [`milestone`] module;
//! this file owns only the state and the hook wiring. No core file is edited to
//! add this — `omm-modules` discovers and links it in.

mod milestone;

use std::any::Any;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use omm_module_api::{
    declare_module, ChatCtx, ItemUseCtx, LevelUpCtx, Module, ModuleManifest, PlayerLoginCtx,
    ServerHooks, TickCtx, ZoneEnterCtx,
};

pub use milestone::{earned, Counts, Milestone};

/// How often (in ticks) the idle heartbeat trace fires — once a minute at the
/// shard's 30 Hz, so a quiet shard still shows the module is alive without
/// flooding logs.
const HEARTBEAT_TICKS: u64 = 1800;

/// The module's live state. Every field is an atomic because hooks take `&self`
/// on one instance shared across the tick thread; the counters only grow, and
/// `unlocked` is a bitmask of the [`Milestone`]s earned so far.
#[derive(Debug, Default)]
pub struct Milestones {
    /// Player logins observed.
    logins: AtomicU64,
    /// Level-up events observed (distinct from [`Milestones::highest_level`]).
    level_ups: AtomicU64,
    /// Zone entries observed.
    zone_enters: AtomicU64,
    /// Chat lines observed.
    chat_lines: AtomicU64,
    /// Item uses observed.
    item_uses: AtomicU64,
    /// Highest character level reached — drives the level milestones.
    highest_level: AtomicU32,
    /// Bitmask of unlocked milestones (see [`Milestone::bit`]).
    unlocked: AtomicU32,
}

impl Milestones {
    /// Snapshot the current counters into a [`Counts`] for [`earned`].
    ///
    /// The loads are independent, so the snapshot can be a hair stale under
    /// concurrent hooks — harmless here: counters are monotonic and thresholds
    /// use `>=`, so the worst case is a milestone recognised one event later,
    /// never falsely granted.
    fn snapshot(&self) -> Counts {
        Counts {
            logins: self.logins.load(Ordering::Relaxed),
            highest_level: u64::from(self.highest_level.load(Ordering::Relaxed)),
            zone_enters: self.zone_enters.load(Ordering::Relaxed),
            chat_lines: self.chat_lines.load(Ordering::Relaxed),
            item_uses: self.item_uses.load(Ordering::Relaxed),
        }
    }

    /// Recompute earned milestones, OR them into `unlocked`, and log each bit
    /// that flipped on for the first time. Idempotent: re-running with unchanged
    /// counters sets no new bit and logs nothing.
    fn reconcile(&self) {
        let mask = earned(&self.snapshot());
        let prev = self.unlocked.fetch_or(mask, Ordering::Relaxed);
        let newly = mask & !prev;
        if newly == 0 {
            return;
        }
        for m in Milestone::ALL {
            if newly & m.bit() != 0 {
                tracing::info!(
                    target: "module::milestones",
                    milestone = m.label(),
                    "milestone unlocked",
                );
            }
        }
    }

    /// The number of milestones unlocked so far.
    #[must_use]
    pub fn unlocked_count(&self) -> u32 {
        self.unlocked.load(Ordering::Relaxed).count_ones()
    }

    /// Whether `milestone` has been unlocked.
    #[must_use]
    pub fn is_unlocked(&self, milestone: Milestone) -> bool {
        self.unlocked.load(Ordering::Relaxed) & milestone.bit() != 0
    }

    /// The highest character level this module has seen.
    #[must_use]
    pub fn highest_level(&self) -> u32 {
        self.highest_level.load(Ordering::Relaxed)
    }

    /// Player logins observed since boot.
    #[must_use]
    pub fn logins(&self) -> u64 {
        self.logins.load(Ordering::Relaxed)
    }

    /// Level-up events observed since boot.
    #[must_use]
    pub fn level_ups(&self) -> u64 {
        self.level_ups.load(Ordering::Relaxed)
    }

    /// Zone entries observed since boot.
    #[must_use]
    pub fn zone_enters(&self) -> u64 {
        self.zone_enters.load(Ordering::Relaxed)
    }

    /// Chat lines observed since boot.
    #[must_use]
    pub fn chat_lines(&self) -> u64 {
        self.chat_lines.load(Ordering::Relaxed)
    }

    /// Item uses observed since boot.
    #[must_use]
    pub fn item_uses(&self) -> u64 {
        self.item_uses.load(Ordering::Relaxed)
    }
}

impl ServerHooks for Milestones {
    fn on_player_login(&self, _ctx: &PlayerLoginCtx) {
        self.logins.fetch_add(1, Ordering::Relaxed);
        self.reconcile();
    }

    fn on_level_up(&self, ctx: &LevelUpCtx) {
        self.level_ups.fetch_add(1, Ordering::Relaxed);
        self.highest_level
            .fetch_max(u32::from(ctx.to.get()), Ordering::Relaxed);
        self.reconcile();
    }

    fn on_zone_enter(&self, _ctx: &ZoneEnterCtx) {
        self.zone_enters.fetch_add(1, Ordering::Relaxed);
        self.reconcile();
    }

    fn on_chat(&self, _ctx: &ChatCtx<'_>) {
        self.chat_lines.fetch_add(1, Ordering::Relaxed);
        self.reconcile();
    }

    fn on_item_use(&self, _ctx: &ItemUseCtx) {
        self.item_uses.fetch_add(1, Ordering::Relaxed);
        self.reconcile();
    }

    fn on_tick(&self, ctx: &TickCtx) {
        if ctx.tick.0 % HEARTBEAT_TICKS == 0 {
            tracing::trace!(
                target: "module::milestones",
                tick = ctx.tick.0,
                unlocked = self.unlocked_count(),
                "milestones heartbeat",
            );
        }
    }
}

impl Module for Milestones {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest::new("milestones", env!("CARGO_PKG_VERSION"))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Emit the `module()` entry point the generated `omm-modules` registry links to.
declare_module!(Milestones::default());

// The full per-hook unlock matrix (explorer/chatterbox/tinkerer, full-session)
// lives in tests/hooks.rs, driving the module through its public API. These
// inline tests cover the lock-free essentials: idempotency and the heartbeat.
#[cfg(test)]
mod tests {
    use omm_ecs_core::EntityId;
    use omm_module_api::Level;
    use omm_protocol::{AccountId, CharacterId, Tick};

    use super::*;

    fn login_ctx() -> PlayerLoginCtx {
        PlayerLoginCtx::new(AccountId::new(1), CharacterId::new(2), EntityId(3))
    }

    fn level_ctx(from: u16, to: u16) -> LevelUpCtx {
        LevelUpCtx::new(
            EntityId(3),
            CharacterId::new(2),
            Level::new(from),
            Level::new(to),
        )
    }

    #[test]
    fn first_login_unlocks_once_and_is_idempotent() {
        let m = Milestones::default();
        assert!(!m.is_unlocked(Milestone::FirstLogin));
        m.on_player_login(&login_ctx());
        m.on_player_login(&login_ctx());
        assert!(m.is_unlocked(Milestone::FirstLogin));
        assert_eq!(m.logins(), 2);
        // Two logins, still exactly one milestone earned.
        assert_eq!(m.unlocked_count(), 1);
    }

    #[test]
    fn level_up_tracks_highest_and_unlocks_at_thresholds() {
        let m = Milestones::default();
        m.on_level_up(&level_ctx(1, 9));
        assert_eq!(m.highest_level(), 9);
        assert!(!m.is_unlocked(Milestone::Reached10));

        m.on_level_up(&level_ctx(9, 10));
        assert!(m.is_unlocked(Milestone::Reached10));
        assert!(!m.is_unlocked(Milestone::Reached20));

        m.on_level_up(&level_ctx(10, 20));
        assert!(m.is_unlocked(Milestone::Reached20));
        assert_eq!(m.highest_level(), 20);
        assert_eq!(m.level_ups(), 3);
    }

    #[test]
    fn highest_level_never_decreases() {
        let m = Milestones::default();
        m.on_level_up(&level_ctx(1, 15));
        // A lower later report never lowers the recorded peak.
        m.on_level_up(&level_ctx(15, 12));
        assert_eq!(m.highest_level(), 15);
    }

    #[test]
    fn tick_heartbeat_leaves_unlock_state_untouched() {
        let m = Milestones::default();
        m.on_tick(&TickCtx::new(Tick(HEARTBEAT_TICKS), 1.0 / 30.0));
        m.on_tick(&TickCtx::new(Tick(1), 1.0 / 30.0));
        assert_eq!(m.unlocked_count(), 0);
    }

    #[test]
    fn module_entry_point_reports_manifest() {
        let boxed = module();
        assert_eq!(boxed.manifest().name, "milestones");
        assert!(boxed.manifest().is_compatible());
    }
}
