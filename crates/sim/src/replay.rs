//! Input capture and deterministic replay from genesis.
//!
//! An [`InputLog`] is the tick-ordered record of every validated intent the
//! server applied to a [`World`]. Pair it with the genesis world (the state
//! before the first [`World::step`]) and the content ability table, and
//! [`replay`] reconstructs the authoritative state **bit-for-bit** — because
//! [`World::step`] is deterministic, the same ordered inputs fold to the same
//! state on any box.
//!
//! That is the executable form of the sim's core promise, and it powers three
//! things at once:
//! - **Server-side replay** — reproduce a session from its input stream.
//! - **Anti-cheat re-simulation** — re-run a suspect's inputs on a trusted box
//!   and compare [`WorldHash`](crate::WorldHash)es; a mismatch is a rejected
//!   client-asserted state.
//! - **Regression capture** — a failing tick becomes a checked-in log.
//!
//! # What the log captures — and what it does not
//!
//! An [`InputLog`] records *intents*, never spawns or authoritative state (the
//! client sends intent, never state — that rule holds here too). Replay folds
//! those intents over the **genesis** world, so genesis must already contain
//! every entity the log references. A `Move` naming an absent entity is a
//! no-op and a cast on an absent target is rejected — exactly as in the live
//! tick — so a log that outruns its genesis degrades safely rather than
//! diverging silently.
//!
//! # Determinism obligations on the caller
//!
//! - **Record in application order.** The live loop applies a tick's
//!   [`InputBatch`](crate::InputBatch) in server-issued id order (the batch
//!   invariant); record it in that same order and replay reproduces it.
//! - **Record every applied intent, and only applied ones.** A rejected cast
//!   still consumed nothing, so it is safe to log; an intent dropped before
//!   [`World::step`] must not be.
//!
//! [`World`]: crate::World
//! [`World::step`]: crate::World::step

use crate::World;
use omm_ecs_core::{AbilityDef, AbilityId, EntityId};
use omm_protocol::{Intent, Tick};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A tick-ordered capture of every validated intent applied to a world.
///
/// Entries are `(tick, entity, intent)` triples in the order the server applied
/// them: non-decreasing by tick, and within a tick in the batch's canonical
/// server-issued id order. [`replay`] folds them back over a genesis world.
///
/// A log also remembers the last tick it *covers* — the final tick a batch was
/// recorded for, even an **empty** one. That is what lets [`replay`] reproduce
/// trailing idle ticks (a session that ends while auras are still ticking); the
/// span, not just the last intent, defines where replay stops. Record an idle
/// tick with [`record_batch`] and an empty batch to include it.
///
/// The record is serializable so a session can be persisted, shipped to a
/// verifier, or checked into a regression suite. Server-issued [`EntityId`]s
/// cross that boundary as their raw `u64` — `omm-ecs-core` is the pure runtime
/// shape and carries no `serde`, so the mapping lives here, at the edge.
///
/// [`record_batch`]: InputLog::record_batch
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InputLog {
    #[serde(with = "entries_serde")]
    entries: Vec<(Tick, EntityId, Intent)>,
    /// The last tick a batch was recorded for (inclusive), empty batches
    /// included; the tick [`replay`] steps through. `None` until anything is
    /// recorded.
    through: Option<Tick>,
}

impl InputLog {
    /// An empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one applied intent at `tick`.
    ///
    /// Must be called in non-decreasing tick order (and in id order within a
    /// tick) to preserve the replay-order invariant; a debug build asserts it.
    pub fn record(&mut self, tick: Tick, entity: EntityId, intent: Intent) {
        self.cover(tick);
        self.entries.push((tick, entity, intent));
    }

    /// Record a whole tick's [`InputBatch`](crate::InputBatch), in batch order.
    ///
    /// Also extends the log's covered span to `tick` **even when `batch` is
    /// empty**, so recording every simulated tick — idle ones included — lets
    /// [`replay`] reproduce the session's final state exactly.
    pub fn record_batch(&mut self, tick: Tick, batch: &[(EntityId, Intent)]) {
        self.cover(tick);
        for (entity, intent) in batch {
            self.entries.push((tick, *entity, intent.clone()));
        }
    }

    /// Extend the covered span to `tick`, upholding the non-decreasing invariant.
    fn cover(&mut self, tick: Tick) {
        debug_assert!(
            self.through.is_none_or(|last| tick >= last),
            "InputLog ticks must be recorded in non-decreasing order",
        );
        self.through = Some(self.through.map_or(tick, |last| last.max(tick)));
    }

    /// The recorded entries, in application order.
    #[must_use]
    pub fn entries(&self) -> &[(Tick, EntityId, Intent)] {
        &self.entries
    }

    /// Number of recorded intents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no intents were recorded. A log that covers only idle ticks (empty
    /// batches) is still empty by this measure — use [`last_tick`] for coverage.
    ///
    /// [`last_tick`]: InputLog::last_tick
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The last tick this log covers — the tick [`replay`] steps up to and
    /// including — or `None` when nothing has been recorded. This is the last
    /// tick a batch was recorded for, which may be later than the last tick that
    /// carries an intent when the session ended on idle ticks.
    #[must_use]
    pub fn last_tick(&self) -> Option<Tick> {
        self.through
    }
}

/// Fold `log` over a clone of `genesis`, reconstructing the authoritative world.
///
/// Steps the world once per tick from `genesis.now()` through the log's last
/// recorded tick — applying each tick's batch, and an **empty** batch for a tick
/// that recorded nothing. Idle ticks are never skipped: auras and cooldowns
/// still advance on them, so skipping would diverge. Deterministic — the same
/// `genesis`, `log`, and `abilities` always fold to the same [`World`].
///
/// `abilities` is the content ability table casts resolve against; it is passed
/// in rather than stored in the log because content is data (a re-sim uses the
/// same pack the live tick did). An empty log yields an unchanged clone of
/// `genesis`.
#[must_use]
pub fn replay(
    genesis: &World,
    log: &InputLog,
    abilities: &BTreeMap<AbilityId, AbilityDef>,
) -> World {
    let mut world = genesis.clone();
    let Some(last) = log.last_tick() else {
        return world; // Nothing recorded — genesis is already the final state.
    };

    let mut cursor = log.entries().iter().peekable();
    // Genesis may start past the log's opening ticks; drop entries the world has
    // already advanced beyond, since step() can never reach them again.
    while cursor
        .peek()
        .is_some_and(|(tick, _, _)| *tick < world.now())
    {
        cursor.next();
    }

    while world.now() <= last {
        let tick = world.now();
        let mut batch: Vec<(EntityId, Intent)> = Vec::new();
        while let Some((entry_tick, entity, intent)) = cursor.peek() {
            if *entry_tick != tick {
                break;
            }
            batch.push((*entity, intent.clone()));
            cursor.next();
        }
        world.step(&batch, abilities);
    }
    world
}

/// `serde` bridge for the entry list: server-issued [`EntityId`]s are the pure
/// runtime type (`omm-ecs-core` carries no `serde`), so each crosses the wire as
/// its raw `u64` and is rewrapped on the way back.
mod entries_serde {
    use omm_ecs_core::EntityId;
    use omm_protocol::{Intent, Tick};
    use serde::ser::SerializeSeq;
    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<S>(
        entries: &[(Tick, EntityId, Intent)],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(entries.len()))?;
        for (tick, entity, intent) in entries {
            seq.serialize_element(&(*tick, entity.raw(), intent))?;
        }
        seq.end()
    }

    pub(super) fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Vec<(Tick, EntityId, Intent)>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Vec::<(Tick, u64, Intent)>::deserialize(deserializer)?;
        Ok(raw
            .into_iter()
            .map(|(tick, id, intent)| (tick, EntityId::new(id), intent))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::Actor;
    use crate::InputBatch;
    use omm_ecs_core::{AuraSpec, EffectKind, Periodic, TargetKind, TargetShape, Team};
    use omm_protocol::{CharacterId, Vec3};
    use proptest::prelude::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn actor(x: f32, z: f32, team: u16, hp: u32) -> Actor {
        Actor::new(at(x, z), Team(team), hp, hp)
    }

    fn move_to(dir: Vec3) -> Intent {
        Intent::Move { dir }
    }

    fn cast_on(id: u32, target: EntityId) -> Intent {
        Intent::UseAbility {
            id,
            target: Some(CharacterId::new(target.raw())),
        }
    }

    fn table(defs: impl IntoIterator<Item = AbilityDef>) -> BTreeMap<AbilityId, AbilityDef> {
        defs.into_iter().map(|d| (d.id, d)).collect()
    }

    /// Instant single-target nuke — no cooldown/GCD, range so wide it always hits.
    fn nuke(id: u32, damage: u32) -> AbilityDef {
        AbilityDef {
            id: AbilityId(id),
            power_cost: 0,
            cooldown_ticks: 0,
            gcd_ticks: 0,
            range: 999.0,
            target_kind: TargetKind::Enemy,
            shape: TargetShape::Single,
            effects: vec![EffectKind::Damage(damage)],
        }
    }

    /// A DoT: applies a periodic-damage aura that ticks on later (idle) ticks.
    fn dot(id: u32, per_tick: u32, duration: u32) -> AbilityDef {
        AbilityDef {
            effects: vec![EffectKind::ApplyAura(AuraSpec {
                period_ticks: 1,
                duration_ticks: duration,
                periodic: Periodic::Damage(per_tick),
            })],
            ..nuke(id, 0)
        }
    }

    /// A scripted per-tick input program run against a fresh world. Returns the
    /// genesis (pre-step) world, the recorded log, and the live final world.
    fn live_run(
        spawns: &[(f32, f32, u16, u32)],
        program: &[Vec<(usize, Intent)>], // per tick: (spawn index, intent)
        abilities: &BTreeMap<AbilityId, AbilityDef>,
    ) -> (World, InputLog, World) {
        let mut world = World::new();
        let ids: Vec<EntityId> = spawns
            .iter()
            .map(|&(x, z, team, hp)| world.spawn(actor(x, z, team, hp)))
            .collect();
        let genesis = world.clone();

        let mut log = InputLog::new();
        for tick_inputs in program {
            let tick = world.now();
            let mut batch: InputBatch = tick_inputs
                .iter()
                .map(|(idx, intent)| (ids[*idx], intent.clone()))
                .collect();
            batch.sort_by_key(|(id, _)| *id); // canonical batch order
            log.record_batch(tick, &batch);
            world.step(&batch, abilities);
        }
        (genesis, log, world)
    }

    fn hashes_match(a: &World, b: &World) -> bool {
        a.state_hash() == b.state_hash() && a.now() == b.now()
    }

    #[test]
    fn empty_log_replays_to_an_unchanged_genesis() {
        let mut world = World::new();
        world.spawn(actor(2.0, 5.0, 1, 100));
        let genesis = world.clone();
        let out = replay(&genesis, &InputLog::new(), &BTreeMap::new());
        assert!(hashes_match(&out, &genesis));
        assert_eq!(out.now(), Tick(0));
    }

    #[test]
    fn replay_reproduces_a_single_move() {
        let program = vec![vec![(0usize, move_to(at(3.0, -6.0)))]];
        let (genesis, log, live) = live_run(&[(0.0, 0.0, 1, 100)], &program, &BTreeMap::new());
        let out = replay(&genesis, &log, &BTreeMap::new());
        assert!(hashes_match(&out, &live));
        assert_eq!(out.now(), Tick(1));
    }

    #[test]
    fn replay_matches_a_multi_entity_move_and_cast_run() {
        let abilities = table([nuke(1, 15)]);
        let spawns = [(0.0, 0.0, 1, 100), (1.0, 0.0, 2, 100), (-2.0, 3.0, 1, 100)];
        // Entity 0 casts at entity 1; entities 0 and 2 also move around.
        let program = vec![
            vec![
                (0, cast_on(1, EntityId::new(2))),
                (2, move_to(at(1.0, 1.0))),
            ],
            vec![(0, move_to(at(-1.0, 0.0))), (2, move_to(at(0.0, -2.0)))],
            vec![(0, cast_on(1, EntityId::new(2)))],
            vec![],
            vec![(2, move_to(at(4.0, 4.0)))],
        ];
        let (genesis, log, live) = live_run(&spawns, &program, &abilities);
        let out = replay(&genesis, &log, &abilities);
        assert!(hashes_match(&out, &live));
        assert_eq!(live.len(), 3, "no one died in this run");
    }

    #[test]
    fn replay_steps_through_idle_ticks_so_dots_land() {
        // A DoT applied on tick 0, then four idle ticks. If replay skipped the
        // empty ticks the aura would never fire and the hash would diverge.
        let abilities = table([dot(1, 5, 8)]);
        let spawns = [(0.0, 0.0, 1, 100), (1.0, 0.0, 2, 100)];
        let mut program = vec![vec![(0usize, cast_on(1, EntityId::new(2)))]];
        program.extend((0..4).map(|_| Vec::new())); // idle ticks tick the aura
        let (genesis, log, live) = live_run(&spawns, &program, &abilities);

        assert!(
            live.get(EntityId::new(2)).unwrap().health.current < 100,
            "the DoT must have ticked during the idle ticks",
        );
        let out = replay(&genesis, &log, &abilities);
        assert!(hashes_match(&out, &live));
        assert_eq!(out.now(), Tick(5));
    }

    #[test]
    fn replay_reproduces_a_lethal_cast_and_its_prune() {
        let abilities = table([nuke(1, 30)]);
        let spawns = [(0.0, 0.0, 1, 100), (1.0, 0.0, 2, 20)]; // target has 20 hp
        let program = vec![vec![(0usize, cast_on(1, EntityId::new(2)))]];
        let (genesis, log, live) = live_run(&spawns, &program, &abilities);
        assert_eq!(live.len(), 1, "the target was pruned after dying");
        let out = replay(&genesis, &log, &abilities);
        assert!(hashes_match(&out, &live));
        assert!(out.get(EntityId::new(2)).is_none());
    }

    #[test]
    fn replay_is_deterministic() {
        let abilities = table([nuke(1, 7)]);
        let spawns = [(0.0, 0.0, 1, 100), (5.0, 0.0, 2, 100)];
        let program = vec![
            vec![(0usize, cast_on(1, EntityId::new(2)))],
            vec![(0usize, move_to(at(2.0, 2.0)))],
        ];
        let (genesis, log, _) = live_run(&spawns, &program, &abilities);
        let a = replay(&genesis, &log, &abilities);
        let b = replay(&genesis, &log, &abilities);
        assert_eq!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn serde_roundtrip_preserves_the_log_and_its_replay() {
        let abilities = table([nuke(1, 9), dot(2, 3, 6)]);
        let spawns = [(0.0, 0.0, 1, 100), (2.0, 0.0, 2, 100)];
        let program = vec![
            vec![(0usize, cast_on(2, EntityId::new(2)))], // DoT
            vec![(0usize, move_to(at(1.0, 0.0)))],
            vec![],
            vec![(0usize, cast_on(1, EntityId::new(2)))], // nuke
        ];
        let (genesis, log, live) = live_run(&spawns, &program, &abilities);

        let json = serde_json::to_string(&log).unwrap();
        let restored: InputLog = serde_json::from_str(&json).unwrap();
        assert_eq!(log, restored, "log survives a serde roundtrip unchanged");

        let out = replay(&genesis, &restored, &abilities);
        assert!(
            hashes_match(&out, &live),
            "the deserialized log replays identically"
        );
    }

    #[test]
    fn record_maintains_order_and_accessors() {
        let mut log = InputLog::new();
        assert!(log.is_empty());
        assert_eq!(log.last_tick(), None);

        log.record(Tick(0), EntityId::new(1), move_to(at(1.0, 0.0)));
        log.record_batch(
            Tick(2),
            &[
                (EntityId::new(1), move_to(at(0.0, 1.0))),
                (EntityId::new(2), move_to(at(2.0, 0.0))),
            ],
        );
        assert_eq!(log.len(), 3);
        assert!(!log.is_empty());
        assert_eq!(log.last_tick(), Some(Tick(2)));
        assert_eq!(log.entries()[0].0, Tick(0));
        assert_eq!(log.entries()[2].1, EntityId::new(2));
    }

    #[test]
    fn covered_span_extends_past_the_last_intent() {
        // Recording an idle tick's empty batch extends replay coverage without
        // adding an intent — the session-ends-on-idle-ticks case.
        let mut log = InputLog::new();
        log.record(Tick(0), EntityId::new(1), move_to(at(1.0, 0.0)));
        log.record_batch(Tick(3), &[]);
        assert_eq!(log.len(), 1, "only one intent was recorded");
        assert!(!log.is_empty());
        assert_eq!(
            log.last_tick(),
            Some(Tick(3)),
            "coverage runs past the last intent",
        );
    }

    #[test]
    fn replay_from_a_non_zero_genesis_ignores_past_entries() {
        // Advance a world a few ticks, snapshot it as genesis, then hand replay a
        // log whose only entry predates the snapshot: it must be a safe no-op.
        let mut world = World::new();
        world.spawn(actor(0.0, 0.0, 1, 100));
        world.step(&[], &BTreeMap::new());
        world.step(&[], &BTreeMap::new());
        let genesis = world.clone();
        assert_eq!(genesis.now(), Tick(2));

        let mut log = InputLog::new();
        log.record(Tick(0), EntityId::new(1), move_to(at(9.0, 9.0)));
        let out = replay(&genesis, &log, &BTreeMap::new());
        assert!(
            hashes_match(&out, &genesis),
            "stale entries cannot rewrite genesis"
        );
    }

    proptest! {
        /// A recorded move stream always replays to the live final hash — the
        /// anti-cheat re-simulation contract over arbitrary inputs.
        #[test]
        fn recorded_moves_replay_to_the_live_hash(
            moves in prop::collection::vec((-8.0f32..8.0, -8.0f32..8.0), 0..40)
        ) {
            let program: Vec<Vec<(usize, Intent)>> =
                moves.iter().map(|&(x, z)| vec![(0usize, move_to(at(x, z)))]).collect();
            let (genesis, log, live) = live_run(&[(0.0, 0.0, 1, 100)], &program, &BTreeMap::new());
            let out = replay(&genesis, &log, &BTreeMap::new());
            prop_assert_eq!(out.state_hash(), live.state_hash());
            prop_assert_eq!(out.now(), live.now());
        }

        /// Serde is lossless for arbitrary logs: deserialize(serialize(log)) == log.
        #[test]
        fn serde_roundtrip_is_lossless(
            entries in prop::collection::vec((0u64..20, 1u64..8, -5.0f32..5.0), 0..32)
        ) {
            let mut log = InputLog::new();
            // Entries must be tick-ordered; sort before recording.
            let mut sorted = entries.clone();
            sorted.sort_by_key(|&(tick, _, _)| tick);
            for (tick, id, x) in sorted {
                log.record(Tick(tick), EntityId::new(id), move_to(at(x, 0.0)));
            }
            let json = serde_json::to_string(&log).unwrap();
            let restored: InputLog = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(log, restored);
        }
    }
}
