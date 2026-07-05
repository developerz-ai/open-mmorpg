# 06 — Modding & Extensibility (the core design goal)

## The split that defines "extensible"
| Stays compiled (core) | Lives in data/scripts (content) |
|---|---|
| combat formulas, netcode, DB layer, anti-cheat | classes, factions, races, zones, quests, items, abilities |

If content is data, operators extend **without forking core**. If it's hardcoded, they fork. This split is the whole ballgame for "best open core."

## Mechanisms
- **ECS** — new race/class = new component data + system logic. No engine recompile.
- **Scripting** — **WASM** (sandboxed, language-agnostic, safe for untrusted operator content) primary; **Lua** for lightweight scripts. Spell effects, quest logic, faction mechanics.
- **Faction system as data** — ship a 2-faction reference impl (Alliance/Horde-equivalent, original names). Faction count/reputation/racial bonuses are config-driven. Operators add faction #3 via config + assets, no recompile.
- **Plugin/mod manifest** — Minecraft-datapack-style: operator drops a folder (assets + scripts + config), server loads it. **Version-locked against a stable core API** so core updates don't break mods.
- **Client mirrors server** — client reads faction/class/asset manifests from server on connect instead of hardcoding content. New content ships with no client patch — just new asset bundles + a manifest entry.

## Escape hatch
Operators can still fork/compile **deeper engine modules** (AzerothCore-style compiled modules) for engine-level changes. Data/script layer covers the common case; the compiled-fork path stays open for advanced cases.
