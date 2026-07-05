# 01 — Monorepo Layout

One product, one repo. **Cargo workspace** (Rust everywhere) + a `content/` tree for data-driven game content + `bin/` DX scripts. Agent reads the root [CLAUDE.md](../../CLAUDE.md) and knows where each part lives.

## Target layout (scaffold toward this)
```
open-mmorpg/
├── Cargo.toml                # workspace root
├── CLAUDE.md  AGENTS.md→     # the project brain (symlinked)
├── bin/                      # setup · dev · check · fmt  (DX trio)
├── crates/                   # shared, single-responsibility libraries
│   ├── protocol/             # wire format + message types (server↔client), serde/bincode
│   ├── ecs-core/             # shared components + systems (Bevy ECS), no I/O
│   ├── sim/                  # deterministic game simulation (combat, movement, abilities)
│   ├── netcode/              # UDP reliability, snapshot/delta, interest management
│   ├── persistence/          # YugabyteDB access (sqlx), migrations, transactional ownership
│   ├── cache/                # Dragonfly access — ephemeral/broadcast only
│   ├── scripting/            # WASM + Lua host, sandbox, mod manifest loader
│   ├── content-schema/       # typed schema for factions/classes/items/quests (validates content/)
│   └── errors/               # shared typed error enums (thiserror)
├── apps/                     # deployable binaries
│   ├── gateway/              # axum: auth, routing, DDoS edge, session tokens
│   ├── shard/                # headless zone server (tokio + ecs-core + netcode + sim)
│   ├── client/               # Bevy game client
│   ├── mcp/                  # per-account companion MCP server (07)
│   ├── worldsvc/             # cross-shard services: chat, guild, auction house, world feed
│   └── web/                  # operator website — Bun + SolidJS (only non-Rust app, 09)
├── packages/                 # TS/Solid shared code for apps/web (i18n, ui) — Bun workspace
│   ├── i18n/                 # translator · catalog · locales · interpolate (SRP modules)
│   └── ui/                   # shared Solid components, dark-theme tokens
├── content/                  # DATA-DRIVEN game content (no recompile to change)
│   ├── factions/  classes/  races/  abilities/  quests/  zones/
│   ├── scripts/              # WASM/Lua behavior scripts
│   └── manifest.json         # mod/datapack manifest, version-locked to core API
├── assets/                   # glTF / heightmaps / textures / audio (AI-generated, zstd bundles)
├── docker/                   # compose: Yugabyte + Dragonfly + gateway/shard
└── docs/                     # this + initial-idea; ADRs in docs/architecture/decisions/
```

## Rules
- **`ecs-core` and `content-schema` are pure** — types + logic, no I/O. Keeps the dep graph acyclic.
- **`persistence` is the only crate that writes ownership.** `cache` cannot. Enforce with types, not discipline ([04](04-data-and-consistency.md)).
- **`sim` is deterministic** — same inputs → same state, for lockstep/replay/anti-cheat validation.
- **One crate = one reason to change.** File ≤300 LOC (Rust gold-standard).
- **Content never requires a recompile.** If changing a faction needs `cargo build`, the split is wrong ([05](05-ecs-and-scripting.md)).

## Best-tool-per-part
Rust is the default; the monorepo lets a part use a better tool when it genuinely needs one. The **one committed exception is `apps/web`** — the operator website in **Bun + SolidJS**, because it's a standard web product, not a hot path ([09](09-operator-web.md)). It lives under `apps/` behind the same `bin/` scripts and CI. Don't add further runtimes without a reason.
