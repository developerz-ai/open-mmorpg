# Classes

> The **13** ways to be a hero of [Auralon](../01-world-and-cosmology.md). Each class is a fantasy + an [armor](../itemization.md) weight + a resource + a home among the [six forces](../01-world-and-cosmology.md#the-six-cosmic-forces). Abilities are **data** ([content-scripting](../../game-server/content-scripting/README.md)); the [combat sim](../../game-server/combat/README.md) resolves them, compiled & deterministic. Roles are the **holy trinity, kept** — Tank / Healer / DPS, legible on purpose ([pillars](../00-design-pillars.md)).

## The roster
| Class | Fantasy | Armor | Roles | Resource | Force |
|---|---|---|---|---|---|
| **Vanguard** | Frontline martial soldier | Plate | Tank · DPS | **Fury** | Martial |
| **Templar** | Armored holy warrior | Plate | Tank · Healer · DPS | **Zeal** + mana | Dawn |
| **Revenant** ⚔ | Death-bound knight | Plate | Tank · DPS | **Runic power** + runes | Hollow |
| **Ranger** | Marksman + beast companion | Mail | DPS | **Focus** | Martial · Bloom |
| **Warden** | Totemic elementalist | Mail | Healer · DPS | mana + totemic | the elements (Bloom/Lattice/Rift) |
| **Shade** | Stealthed assassin | Leather | DPS | **Energy** + combo points | Martial · stealth |
| **Wilder** | Shapeshifting primal | Leather | Tank · Healer · DPS | **Essence** + mana | Bloom |
| **Ascetic** | Inner-force martial artist | Leather | Tank · Healer · DPS | **Ki** | Martial · inner-force |
| **Riftblade** ⚔ | Chaos-touched hunter | Leather | Tank · DPS | **Havoc** | Rift |
| **Arcanist** | Scholar of raw arcane | Cloth | DPS | mana | Lattice |
| **Occultist** | Pact-caster + summons | Cloth | DPS | mana + **soul shards** | Rift · Hollow · Deep |
| **Oracle** | Radiance-healer / void-caller | Cloth | Healer · DPS | mana | Dawn · Deep (Umbra) |
| **Channeler** | Mid-range empowerment mage | Mail | Healer · DPS | **Essence** + mana | Lattice · Bloom |

⚔ = **hero class** (see below).

## Roles & armor
- **3 specializations per class** — one per role the class offers (see [talents](talents-and-hero-trees.md)). 39 specs total; you pick **1** = your role.
- **Tanks (6):** Vanguard, Templar, Revenant, Wilder, Ascetic, Riftblade. **Healers (6):** Templar, Warden, Wilder, Ascetic, Oracle, Channeler. **DPS: every class.** No role is starved; no comp is impossible.
- **Armor weight** (Cloth / Leather / Mail / Plate) is fixed per class — an itemization lane, not a stat wall. Primary stats **Might / Finesse / Focus / Spirit** map to fantasy, not to a locked class list ([itemization](../itemization.md)).
- **Bring the player, not the class** — no raid seat held hostage to one buff ([pillars](../00-design-pillars.md)).

## Hero classes
Two classes gate by **Age**, not by race — you earn them by reaching the Age their story opens ([ages](../progression/ages.md)):
| Class | Unlocks | Age |
|---|---|---|
| **Revenant** | *The Hollow King* | Age III |
| **Riftblade** | *The Rift Crusade* | Age VII |
Others open with lore-linked Ages too — **Ascetic** (Age V, *The Veiled Continent*), **Channeler** (Age X, *The Skywardens*, flavor-linked to the **Sarn** but not race-walled).

## Class & race freedom
Minimal lock ([pillars](../00-design-pillars.md) #14): **nearly any race plays nearly any class.** Restrictions are light lore flavor — a rendered starting theme, a line of dialogue — **never a hard wall**. Both factions ([Concord & Pact](../factions/README.md)) share the **same 13 classes**; a class is an identity across the divide, not a faction gate.

## Distilled from
| WoW thing | What it is | Verdict |
|---|---|---|
| The 13-class trinity model | Tank/heal/dps roster, one class = one fantasy | **Keep** — legible roles, strong class identity |
| Class/race lock matrix | "You can't be that" walls | **Fix** — light flavor only, no hard walls |
| Borrowed-power mastery (artifact/covenant) | Signature power deleted each expansion | **Fix** — mastery is **permanent & additive**, never confiscated at an Age boundary ([talents](talents-and-hero-trees.md)) |
| Hero/DK-style gated classes | Special class unlocked by content | **Keep** the drama — gate by **Age**, not race |

## Rules
- **A class = data + fantasy; resolution = compiled.** New class ships as content, no `cargo build` ([combat](../../game-server/combat/README.md)).
- **3 specs per class, 1 chosen** = your trinity role. Respec is free/cheap ([talents](talents-and-hero-trees.md)).
- **Armor & resource are fixed per class; race is (almost) never.**
- **Signature-weapon mastery is permanent** — earned power stays earned across all 25 Ages ([pillars](../00-design-pillars.md) #1).
- **Original IP only** — every class name, resource, and force is ours ([world](../01-world-and-cosmology.md)).

## Links
[talents & hero trees](talents-and-hero-trees.md) · [combat sim](../../game-server/combat/README.md) · [content-scripting](../../game-server/content-scripting/README.md) · [itemization](../itemization.md) · [factions](../factions/README.md) · [ages](../progression/ages.md) · [rails](../progression/rails.md) · [endgame](../endgame/README.md) · [design pillars](../00-design-pillars.md) · [world](../01-world-and-cosmology.md) · [CLAUDE](../../../../CLAUDE.md)
