# Talents & Hero Trees

> How a [class](README.md) grows across the 25 [Ages](../progression/ages.md). Three layers — **Specialization** (your role), **Talent trees** (your build), **Hero trees** (your permanent flavor). Every node is **data** ([content-scripting](../../game-server/content-scripting/README.md)); the compiled [combat sim](../../game-server/combat/README.md) resolves what the points buy. The reversal ([pillars](../00-design-pillars.md) #1, #6): choices are **genuinely divergent**, and nothing you earn is ever deleted.

## The three layers
| Layer | What you pick | Cadence | Reversible? |
|---|---|---|---|
| **1 · Specialization** | 1 of 3 per class = your trinity **role** (Tank / Healer / DPS) | early, then swap freely | yes — free/cheap respec |
| **2 · Talent trees** | points in a **class tree** + a **spec tree** (point-buy branching graphs) | every level up the 1→300 arc | yes — freely respec'd |
| **3 · Hero trees** | 1 of **2** hero paths per spec — permanent, additive flavor | unlocks mid-arc, grows across Ages | path swaps freely; **power is never confiscated** |

## Layer 1 — Specialization
Pick **1 of 3**. It sets your role, your core kit, and which primary stat ([Might/Finesse/Focus/Spirit](../itemization.md)) you scale on. Swapping specs is a **free/cheap respec**, and every class gets **dual-spec-or-better** — carry a tank build and a dps build, switch out of combat. No re-grind, ever ([pillars](../00-design-pillars.md) #9).

## Layer 2 — Talent trees
Two graphs, Dragonflight-style point-buy:
- **Class tree** — shared across all 3 specs. Utility, defensives, mobility, identity.
- **Spec tree** — the chosen spec's signature power and rotation.
- **Branching, gated by point-spend**, with **choice nodes** (pick one of two effects). Designed so no single build strictly dominates — nodes are **situational** (single-target vs. cleave, mobility vs. throughput, survival vs. output). A sim can't solve one "correct" tree ([pillars](../00-design-pillars.md) #6).
- **Respec is free/cheap, anywhere out of combat.** Loadouts save; swap per encounter.

## Layer 3 — Hero trees
War-Within-style **hero talents**: each spec chooses **1 of 2 thematic paths** that fuse the class fantasy with a [cosmic force](../01-world-and-cosmology.md#the-six-cosmic-forces) (e.g. a Templar leaning harder into **Dawn**, or a Revenant deeper into **Hollow**). A path is a small tree of **permanent, additive** nodes.
- **The #1 rule: hero power is never deleted at an Age boundary.** What you earn in Age III still works in Age XXV — additive, never reset ([pillars](../00-design-pillars.md) #1). This is the direct reversal of borrowed-power treadmills.
- You may **swap which path** you run freely (it's a build choice), but the *system itself* is never confiscated and re-issued each expansion.

## Talents are data
A talent node is a **content asset**, not code:
- A node grants/modifies [**AbilityDef**s](../../game-server/combat/README.md) — new abilities, changed costs, added effects, aura tweaks — all validated [`content-schema`](../../game-server/content-scripting/README.md).
- The **compiled sim** applies them deterministically on the shard; the client renders intent, never authors state ([combat](../../game-server/combat/README.md)).
- New talent, new hero path, new spec = **new data, no `cargo build`** ([CLAUDE](../../../../CLAUDE.md)).

## Distilled from
| WoW era | What it did | Verdict |
|---|---|---|
| **Vanilla** 3-tree talents | Deep trees, but most points mandatory filler | **Keep** the tree feel; **fix** the illusion-of-choice |
| **Cataclysm** pruned trees | Fewer points, "here's your build" | **Fix** — restore real breadth |
| **MoP** talent rows | 1-of-3 per tier — tidy but shallow, one row-winner | **Fix** — rows solved themselves; go back to graphs |
| **Dragonflight** class+spec trees | Return to real branching point-buy, free respec | **Keep** — this is the model |
| **The War Within** hero talents | Permanent cross-spec flavor paths | **Keep** the structure; **fix** the borrowed-power reset — ours **persists across every Age** |

## Rules
- **Every node is data**; the compiled sim resolves it ([combat](../../game-server/combat/README.md), [content-scripting](../../game-server/content-scripting/README.md)).
- **Respec is free/cheap** and out-of-combat; **dual-spec-or-better** for all classes ([pillars](../00-design-pillars.md) #9).
- **Trees must be genuinely divergent** — situational nodes, no strictly-dominant build ([pillars](../00-design-pillars.md) #6).
- **Hero power is permanent & additive** — nothing earned is deleted at an Age boundary ([pillars](../00-design-pillars.md) #1).
- **Original IP only** — node names, paths, and forces are ours ([world](../01-world-and-cosmology.md)).

## Links
[classes](README.md) · [combat sim](../../game-server/combat/README.md) · [content-scripting](../../game-server/content-scripting/README.md) · [itemization](../itemization.md) · [ages](../progression/ages.md) · [rails](../progression/rails.md) · [endgame](../endgame/README.md) · [design pillars](../00-design-pillars.md) · [world & cosmology](../01-world-and-cosmology.md) · [factions](../factions/README.md) · [CLAUDE](../../../../CLAUDE.md)
