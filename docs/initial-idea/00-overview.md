# 00 — Project Overview

## Vision
WoW-inspired MMORPG, built better, **fully original IP** (no ripped assets, no legal exposure). Open-core, AI-first. Gamers-first, not extraction-first. Best-looking graphics, fastest server, greatest DX for the agents that build it.

## Core principles
- **100% original assets** (AI-generated), original names/lore — zero derivative-IP risk.
- **Open source**, MIT — engine + client. Operators fork freely; we *ask* (not force) meaningful improvements come back upstream.
- **Data-driven core** — factions, classes, races, abilities, quests = config/scripts, not hardcoded engine code. Operators extend without forking core.
- **AI-native, two ways**: (1) AI coding agents write the codebase; (2) players get AI companion agents via MCP.
- **Modern, horizontally-scalable core** — newer than any existing private server (TrinityCore/AzerothCore/CMaNGOS) or retail WoW itself.
- **Rust across the stack** — see [03-tech-stack.md](03-tech-stack.md).

## Reference point
Gameplay loop inspired by WoW's two-faction structure (Alliance/Horde-equivalent, original names) at the current WoW Midnight-tier feature bar: player housing, open-world dynamic encounters, solo/small-group endgame (Delves-equivalent), expanded hero-talent trees, social hub, QoL/UX systems. See [08-feature-bar.md](08-feature-bar.md).

## What makes us better (not just "free WoW")
1. **No realm queues** — autoscaled shards, not fixed realm caps.
2. **Seamless zones** — no loading screens by design, not retrofitted.
3. **Server-authoritative housing/social** built clean, not bolted onto 20-year-old code.
4. **Modding as a first-class citizen** (manifest/plugin system), not addon hacks.
5. **Open, modern asset formats** — no proprietary reverse-engineered tooling.
6. **AI companion agents via MCP** — no equivalent in retail WoW.
7. **Reactive, living world** — GTA-inspired NPC/faction AI, weather, world events ([09](09-gta6-inspiration.md)).
