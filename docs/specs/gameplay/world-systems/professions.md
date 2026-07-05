# Professions — gather, craft, bridge the tiers

> Professions are a full [rail](../progression/rails.md): a parallel climb of gathering and crafting with a **knowledge tree** per profession, quality tiers, and specialization points. Their payoff is the one that matters most — **deterministic crafted gear that bridges raid tiers**, the rail that guarantees you're *never* wall-blocked by a drop that won't come ([itemization](../itemization.md), [pillars](../00-design-pillars.md) #13). Knowledge is account-wide ([warband](../progression/rails.md)). → [economy](../../game-server/economy/README.md) for transactional plumbing.

## The profession set (original-flavored, legible)
| Kind | Profession | Archetype |
|---|---|---|
| **Gathering** | **Delving** (ores/gems) · **Foraging** (herbs) · **Reaving** (hides/beast mats) | Mining · Herbalism · Skinning |
| **Crafting** | **Forgecraft** (plate/weapons) · **Runeweaving** (cloth/regalia) · **Leathercraft** (mail/leather) | Smithing · Tailoring · Leatherworking |
| **Crafting** | **Alchemy** (potions/flasks) · **Glyphcraft** (enchants/glyphs) · **Artifice** (gems/gadgets) | Alchemy · Enchanting · Jewelcrafting/Engineering |

## Quality, stats & the knowledge tree (Dragonflight-archetype)
- **Quality tiers** — each craft rolls a quality (I–V); higher tiers = higher iLvl / better finish. Quality is driven by your **crafting stats**, not RNG.
- **Crafting stats** — *Skill* (unlock higher quality), *Precision* (raise floor), *Ingenuity* (resource refunds), *Insight* (better proc yields). Sourced from tools, mats, and the tree.
- **Knowledge / specialization tree** — earn **specialization points** by doing the profession; spend them to deepen a branch (e.g. Forgecraft → *Weapons* vs *Plate*). Branches are genuinely divergent, situational — no single correct build ([pillars](../00-design-pillars.md) #6).
- **Work orders / commissions** — craft for other players (or guilds) against posted orders; commissions and mat provision flow through the [economy](../../game-server/economy/README.md), transactional & idempotent.

## Deterministic crafted gear — the bridge (the headline)
Crafting is the **anti-wall** [rail](../progression/rails.md): a set you can *plan and finish*, not gamble for.
| Problem it fixes | How professions fix it |
|---|---|
| A BiS slot won't drop for weeks | Craft a **deterministic** piece at a known iLvl to fill it |
| New Age, gear gap before first raid | Crafted gear bridges into the tier's [30-level window](../itemization.md) |
| Sockets/tertiaries were a lottery ([itemization](../itemization.md)) | **Crafted or targeted**, never gambled |
| No pity on drops | Craft *is* the pity — effort-metered, guaranteed output |

Crafted pieces slot into the [gear ladder](../itemization.md) at a coherent band: they **fill & bridge**, BiS-adjacent, never trivializing the raid tier that anchors the window.

## Mastery as a rail — account-wide, no treadmill
- Profession knowledge lives on the **[warband](../progression/rails.md)** — earned once, shared across characters. No alt re-grinds the same tree ([pillars](../00-design-pillars.md) #9).
- **Effort-metered, not calendar-gated** — knowledge advances when you craft/gather, not on a weekly reset ([pillars](../00-design-pillars.md) #8). Catch-up is generous.
- Profession power is **small & horizontal** — utility and access, not a second gear-score you must max ([rails](../progression/rails.md)).

## Distilled from
| Source | System | Verdict |
|---|---|---|
| Vanilla WoW | Flat linear professions, grind to a number | **Fix** — shallow, no build, pure time-sink |
| Dragonflight | Quality tiers, crafting stats, spec tree, work orders | **Keep** the depth — it was the pro-player correction ([Age X](../progression/ages.md)) |
| DF bind-on-pickup mats & spark gating | Mat walls that blocked crafting | **Fix** — no BoP mat walls; effort-metered, warband knowledge |
| Titanforging / socket lottery | RNG-on-RNG gear | **Fix** — crafting is deterministic, targetable ([itemization](../itemization.md)) |

## Rules
- **Professions are a [rail](../progression/rails.md)** — permanent, additive, **warband** account-wide knowledge.
- **Crafted gear is deterministic** and **bridges raid tiers** — never wall-blocked by a drop ([itemization](../itemization.md), [pillars](../00-design-pillars.md) #13).
- **Quality is skill-driven, not RNG** — crafting stats and the tree, not a slot machine.
- **Effort-metered, no mat walls** — no BoP gate; catch-up generous ([pillars](../00-design-pillars.md) #8).
- **Ownership = one [persistence](../../game-server/economy/README.md) transaction** — craft/gather/trade transactional & idempotent, never via cache ([CLAUDE.md](../../../../CLAUDE.md)).
- **All recipes & mats are data** — new professions ship with no recompile ([content-scripting](../../game-server/content-scripting/README.md)).

## Links
[world-systems](README.md) · [housing](housing.md) · [living-world](living-world.md) · [itemization](../itemization.md) · [rails](../progression/rails.md) · [ages](../progression/ages.md) · [economy](../../game-server/economy/README.md) · [content-scripting](../../game-server/content-scripting/README.md) · [pillars](../00-design-pillars.md)
