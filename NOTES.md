# open-mmorpg

An open-core MMORPG engine built in Rust — server and client both — with a Bun/SolidJS operator web app alongside. It is MIT-licensed, original IP only, and explicitly AI-native. The design is server-authoritative (clients send intent, never state), horizontally scaled from day one with autoscaled shards and no realm caps, and data-driven: the core is compiled Rust in `crates/`, while content (factions, classes, quests) lives as data in `content/` so changing it requires no recompile. Ownership writes go straight to YugabyteDB in a transaction — never via cache or bus — which is the project's primary anti-dupe rule.

- **Stack:** Rust (server, client via Bevy, tooling, MCP), pinned by `rust-toolchain.toml`; Bun + SolidJS + TypeScript for `apps/web` (dark theme only, i18n from day one, Zod at boundaries). Datastores: YugabyteDB for durable state, Dragonfly for ephemeral. Open asset formats only (glTF, heightmap, zstd). Docker for local infra; Biome + clippy + nextest for the gate.
- **Key commands:** (the CLAUDE.md marks these as the target trio, created with the monorepo scaffold)
  - `bin/setup` — prereqs, deps, docker (Yugabyte + Dragonfly), migrate
  - `bin/dev [target]` — boot gateway/shard/client/web with reload
  - `bin/check` — the gate: fmt-check + `clippy -D warnings` + nextest (+ biome/tsc for web)
  - `bin/fmt` — `cargo fmt` + `biome --write`
  - `cargo nextest run`, `cargo clippy --all-targets -D warnings`, `bun run --filter @omm/web test`
- **Layout:**
  - `crates/` — compiled core: `sim` (deterministic), `ecs-core` + `content-schema` (pure, no I/O), `persistence` (the only crate that writes ownership), `cache`, `protocol` (wire types), `netcode`, `transport`, `world`, `scripting`, `modules`, `errors`
  - `apps/` — `gateway`, `shard`, `worldsvc`, `client`, `mcp`, `web` (SolidJS operator UI)
  - `content/` — data-driven game content; changing it must not require a rebuild
  - `docs/` — `initial-idea/` (legal, tech stack) and `architecture/` (server topology, data and consistency, ECS and scripting, operator web)
  - `packages/`, `modules/`, `assets/`, `scripts/`, `docker/` — TS packages, mod surface, art assets, tooling, local infra
- **State as of 2026-07-21:** on branch `main`; working tree was clean when this note was written.
