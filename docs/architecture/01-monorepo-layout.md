# 01 — Monorepo Layout

One product, one repo. **Cargo workspace: 14 libraries under `crates/`, five Rust apps, and two compiled modules**; a separate **Bun workspace** for the operator web and shared packages; a `content/` datapack; and `bin/` DX scripts. Agent reads the root [CLAUDE.md](../../CLAUDE.md) and knows where each part lives.

## Target layout (scaffold toward this)
The tree below shows checked-out components, not a completed target architecture. Planned features and missing payloads are called out below it.

```
open-mmorpg/
├── Cargo.toml                # workspace root
├── CLAUDE.md  AGENTS.md→     # the project brain (symlinked)
├── bin/                      # setup · dev · check · fmt · new-module  (DX scripts)
├── crates/                   # 14 shared, single-responsibility libraries
│   ├── protocol/             # server↔client message types, serde (no bincode dependency)
│   ├── ecs-core/             # plain component structs + pure systems; no Bevy dependency
│   ├── sim/                  # deterministic game simulation (combat, movement, abilities)
│   ├── netcode/              # reliability, snapshot/delta, interest management, Transport seam
│   ├── transport/            # real socket drivers + connection lifecycle for Transport
│   ├── persistence/          # YugabyteDB access (sqlx), migrations, transactional ownership
│   ├── cache/                # ephemeral-store trait + MemoryCache; Dragonfly backend planned
│   ├── scripting/            # fuel-metered WASM host; Lua and datapack script binding planned
│   ├── content-schema/       # typed schemas + validation; split-tree loader does filesystem I/O
│   ├── world/                # deterministic quadtree: interest, streaming, shard boundaries
│   ├── engine-core/          # Bevy headless App/Plugin bootstrap, reflection, fixed-timestep schedule
│   ├── module-api/           # compiled-module hook traits + ModuleHost (architecture/10-modules.md)
│   ├── modules/              # generated module aggregator (architecture/10-modules.md)
│   └── errors/               # shared typed error enums (thiserror)
├── apps/                     # five Rust apps + separate Bun web app
│   ├── gateway/              # axum edge/auth + session tokens
│   ├── shard/                # headless tick loop + datapack abilities; transport/storage unwired
│   ├── client/               # headless prediction demo using shared sim; headful planned
│   ├── mcp/                  # bounded companion-action scaffold (architecture/07-mcp-companions.md)
│   ├── worldsvc/             # HTTP read/social plane: chat, guilds, market, feed
│   └── web/                  # Bun + SolidJS operator website (architecture/09-operator-web.md)
├── packages/                 # TS/Solid shared code for apps/web (i18n, ui) — Bun workspace
│   ├── i18n/                 # translator · catalog · locales · interpolate (SRP modules)
│   └── ui/                   # shared Solid components, dark-theme tokens
├── content/                  # DATA-DRIVEN game content (no recompile to change)
│   ├── factions/  classes/  races/  abilities/  quests/  zones/
│   ├── items/  spawn-tables/  dungeons/  economy/
│   └── manifest.json         # datapack metadata + content API version; domains loaded recursively
├── modules/                  # two compiled modules (architecture/10-modules.md)
│   ├── hello-world/          # tick/login hook example; tick wired in shard binary
│   └── milestones/           # milestone hook example; gameplay event sources still need wiring
├── assets/                   # metadata/docs only; referenced placeholder payloads absent
│   ├── README.md
│   └── manifest.json
├── docker/                   # compose: Yugabyte + Dragonfly data stores (not gateway/shard)
└── docs/                     # this + initial-idea; ADRs in docs/architecture/decisions/
```

## Implementation boundaries
- **Client:** `apps/client` is a headless prediction demo. The Bevy/wgpu/PBR headful client is **not yet implemented (planned: C5/C8)** — [client batches](../mvp/v1/40-game-client.md#pr-batches-100-prs). `engine-core` already uses Bevy app/ECS/reflection/time subcrates; that is not renderer wiring.
- **Scripting:** the fuel-metered WASM host exists. Lua is **not yet implemented (planned: S9)** — [server batches](../mvp/v1/30-game-server.md#pr-batches-100-prs). `content/scripts/` is planned, not present; datapack-to-script compilation/binding remains a scaffold.
- **Cache:** `MemoryCache` exists; the Dragonfly backend is **not yet implemented (planned: D6)** — [database batches](../mvp/v1/10-database.md#pr-batches-100-prs). Starting Dragonfly with Docker does not implement this backend.
- **Shard wiring:** `transport` supplies a UDP library, but `apps/shard/src/main.rs` does not wire it into the tick loop. Persistence and cache are not shard dependencies today. Autoscaling and seamless handoff remain [S10 targets](../mvp/v1/30-game-server.md#pr-batches-100-prs).
- **Data vs assets:** the committed datapack loads and validates today; the shard loads its ability table at boot. Asset generation, future mesh/heightmap/texture/audio directories, streaming and hot-swapping are planned; only `assets/README.md` and `assets/manifest.json` ship, not the manifest's referenced payloads. Open formats and original IP remain mandatory ([06](06-world-and-assets.md)).
- **Compiled modules:** see [architecture chapter 10](10-modules.md) for the hook API, generated registry and examples; “10” is a chapter reference, not a module count.

## Rules
- **`ecs-core` is pure** — types + logic, no I/O. **`content-schema` has pure schema/validation logic and byte parsing, plus a filesystem loader** (`load_manifest_dir`); the whole crate is not I/O-free.
- **`persistence` is the only crate that writes ownership.** `cache` cannot. Enforce with types, not discipline ([04](04-data-and-consistency.md)).
- **`sim` is deterministic** — same inputs → same state, for lockstep/replay/anti-cheat validation.
- **One crate = one reason to change.** File ≤300 LOC (Rust gold-standard).
- **Content never requires a recompile.** If changing a faction needs `cargo build`, the split is wrong ([05](05-ecs-and-scripting.md)).

## Best-tool-per-part
Rust is the default; the monorepo lets a part use a better tool when it genuinely needs one. The **one committed exception is `apps/web`** — the operator website in **Bun + SolidJS**, because it's a standard web product, not a hot path ([09](09-operator-web.md)). It lives under `apps/` behind the same `bin/` scripts and CI. Don't add further runtimes without a reason.
