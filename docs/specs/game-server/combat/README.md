# Combat & Abilities

> Deterministic, server-authoritative resolution of every ability. Definitions are **data** ([content](../content-scripting/README.md)); resolution is compiled and replayable ([sim](../../../../crates/sim/src/lib.rs)). → [architecture/05](../../../architecture/05-ecs-and-scripting.md)

## What it does
Turns a validated [`Intent::UseAbility`](../../../../crates/protocol/src/lib.rs) into authoritative state change. The engine resolves; the content defines *what* resolves. Same inputs → same outcome, on any box — the basis for replay and anti-cheat re-sim.

## Model (immutable definition + live instance)
Borrowed straight from TrinityCore's genuinely good design, ported to ECS:

| TrinityCore | Ours | Nature |
|---|---|---|
| `SpellInfo` (immutable, from DB2) | **AbilityDef** — validated [`content-schema`](../../../../crates/content-schema/src/lib.rs) asset | data |
| `Spell` (live cast, `m_castId`) | **Cast** — an ECS entity/command with caster, target, tick | runtime |
| `SpellEffectHandlers` dispatch table | **effect systems** keyed by effect kind | compiled |
| `Aura`+`AuraApplication`+`AuraEffect` | **Aura** container + per-target application + per-effect **component** | ECS-native |
| `ThreatManager` | explicit **Threat** component/system | ECS-native |

## Cast pipeline (per ability, deterministic)
1. **Validate** ([security](../security/README.md)): power/resource cost, cooldown + GCD, range, line-of-sight ([world-model](../world-model/README.md) collision), valid target, ownership. Reject → [`ServerMsg::rejected`](../../../../crates/protocol/src/lib.rs), don't crash.
2. **Select targets** — radius/cone/chain geometry from the AbilityDef against AoI.
3. **Take costs** — power/reagents, atomic with the cast.
4. **Resolve effects** — dispatch each effect (`Damage`, `ApplyAura`, `Heal`, …) to its compiled system; parameters come from data. Instant, or delayed by missile travel (resolved at the arrival tick).
5. **Apply** — mutate [`Health`](../../../../crates/ecs-core/src/lib.rs) etc.; generate threat; any item/currency outcome (loot, consume) goes through a [`persistence`](../persistence/README.md) transaction, never in-memory only.

## Auras (buffs/DoTs/HoTs)
- **Aura** = effect container; **AuraApplication** = per-target instance; **AuraEffect** = per-effect component. Periodic effects tick on a fixed period against the sim tick, not wall-clock.
- Stacking/refresh/dispel rules are data on the AbilityDef; the engine enforces them uniformly.

## Determinism & lag
- No RNG without a **seeded, replicated** stream; no wall-clock; no float math that varies by platform on the authoritative path ([tick-loop](../tick-loop/README.md)).
- **Fine tick granularity** — never a coarse batch window. Vanilla WoW's ~400 ms spell-batch bucket caused a decade of "he was already dead" / double-CC complaints; Classic cut it to 10 ms. We resolve at sub-perception granularity so simultaneity is real, not bucketed.
- **Lag compensation** only for hitscan/skillshot abilities (bounded ~200–300 ms rewind of the sim-tick snapshot history); tab-target/ability combat validates against present state ([netcode](../netcode/README.md)).

## Distilled from the references
| Source | Verdict |
|---|---|
| TrinityCore spell/aura/threat | **Keep** immutable-def + live-instance split, effect-handler dispatch, 3-part aura model, explicit threat — all ECS-friendly. **Replace** C++-hardcoded effect handlers with data-registered effect systems; make resolution deterministic & server-authoritative (never trust client cast state). |
| WoW spell batching (400 ms → 10 ms) | **Avoid** any coarse batch window; keep tick resolution below human perception. |
| Netcode lag-comp | **Adopt** rewind for skillshots only. |

## Rules
- Ability **definitions never require a recompile** — new ability = new data + (optionally) a script ([content-scripting](../content-scripting/README.md)). If it needs `cargo build`, the compiled/data line is wrong.
- Every economic side effect of combat (loot, consume, craft) is a [`persistence`](../persistence/README.md) transaction ([economy](../economy/README.md)).
- Resolution is pure and lives in/behind [`sim`](../../../../crates/sim/src/lib.rs) — replayable, re-simulatable.

## Links
[tick-loop](../tick-loop/README.md) · [content-scripting](../content-scripting/README.md) · [security](../security/README.md) · [persistence](../persistence/README.md) · [`crates/sim`](../../../../crates/sim/src/lib.rs)
