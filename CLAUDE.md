# Open-MMORPG

The best **open-core MMORPG engine** · Rust (server + client) · Bun/SolidJS operator web · original IP · MIT · AI-native. Server-authoritative, horizontally scaled, data-driven, moddable.

## Response Rules
- Execute. No preamble, no "I'll start by…", no restating the task.
- Lead with the action or answer. Reasoning after, only if non-obvious.
- Parallel tool calls when independent. Read before speculating.
- Disagree when the user is wrong. State the correction.
- Terse. Fragments OK. Code/commands/paths stay exact.
- End: 1–2 sentence summary max.

## <a name="standards"></a>Standards — read before non-trivial work
This repo follows **developerz-ai gold standards**: `../gold-standards-in-ai/docs/`. Defaults from there apply unless a doc here overrides. Highest-value: `00-philosophy.md`, `stack/rust-apis.md`, `architecture/solid-srp.md`, `writing-for-agents/`.

## Binding decisions (do NOT re-litigate)
1. **Rust everywhere** — server, client (Bevy), tooling, MCP. Exception: `apps/web` is Bun+SolidJS (standard web product). → `docs/initial-idea/03-tech-stack.md`
2. **Server-authoritative, always.** Client sends intent, never state.
3. **Ownership writes → YugabyteDB directly, in a transaction.** Never via cache/bus. #1 anti-dupe rule. → `docs/architecture/04-data-and-consistency.md`
4. **Content is data, core is compiled.** Factions/classes/quests change with no recompile. → `docs/architecture/05-ecs-and-scripting.md`
5. **Open formats only** — glTF / heightmap / zstd. No proprietary tooling.
6. **Original IP only.** Never extract/rip any game asset, name, or table. → `docs/initial-idea/01-legal-and-licensing.md`
7. **Horizontal from day 1.** Autoscaled shards, no realm caps, no queues.

## Architecture
- **Core is compiled (`crates/`), content is data (`content/`).** If changing a faction needs `cargo build`, the split is wrong.
- **`persistence` is the only crate that writes ownership.** `cache` (Dragonfly) has no ownership-write API — the dupe path must not compile.
- **`sim` is deterministic** — same inputs → same state (replay, anti-cheat re-sim).
- **`ecs-core` + `content-schema` are pure** — types + logic, no I/O.
- Wire types live in `crates/protocol` — single source of truth server↔client.
- Topology: edge → `gateway` → `shard`(s) → cross-shard bus + `worldsvc`; state in Yugabyte (durable) + Dragonfly (ephemeral). → `docs/architecture/02-server-topology.md`

## Coding Rules
### Think before coding
- State assumptions. Uncertain → ask, don't guess. Multiple readings → present them.
### Simplicity first
- Minimum code that solves the stated problem. No speculative abstraction/config. No handling of impossible cases.
### Surgical changes
- Touch only what the task requires. Match existing style. Every changed line traces to the request.
### Goal-driven execution
- Convert tasks to verifiable goals: bug → reproducing test → make it pass. Plan multi-step as `step → verify: check`.

## Rust conventions (hard)
- **No `unwrap`/`expect` outside `main`/tests** — a hot-path panic drops every player on the shard.
- **Typed errors** (`thiserror`), stable client codes; never leak credentials/internal detail in errors or logs.
- **Newtypes over primitives** — `AccountId`, `ItemId`, not raw `u64`.
- **Borrow by default** — `&str`/`&[T]` over `String`/`Vec`.
- **Async end-to-end** — `tokio::sync`, never `std::sync::Mutex` on the request/tick path.
- **Files ≤300 LOC. One crate = one reason to change.** SOLID/SRP.
- Make illegal states unrepresentable — the type system is the first anti-cheat layer.

## apps/web conventions (Bun + SolidJS)
- TypeScript strict (no `any`), **Zod at every boundary**, custom typed errors.
- **SRP** — one component/one job, ≤300 LOC; shared UI → `packages/ui`. Thin routes; data in TanStack Query hooks.
- **Dark theme ONLY** — no toggle. Semantic CSS-variable tokens by role, defined once in `:root`. Components use `bg-bg text-fg`, never raw hex.
- **i18n from day one** — every string via `t()`; missing keys render loudly (`⟦key⟧`). Dates/money via `Intl`, not i18n.
- → `docs/architecture/09-operator-web.md`

## Commands
> Pre-scaffold — these are the target trio (create with the monorepo scaffold). Keep local, hook, and CI on the same commands.
```
bin/setup            # prereqs → deps → docker (Yugabyte+Dragonfly) → migrate
bin/dev [target]     # boot gateway/shard/client/web with reload
bin/check            # the gate: fmt-check + clippy -D warnings + nextest (+ biome/tsc for web)
bin/fmt              # cargo fmt + biome --write

cargo nextest run                    # Rust tests (fast)
cargo clippy --all-targets -D warnings
bun run --filter @omm/web test       # web tests
```

## Clone setup (AI agents — read on a fresh checkout)
Local agent tooling is **git-ignored** (per-developer, regenerated — never committed). After `git clone`, bootstrap it yourself; a fresh tree won't have it:
- **`.codegraph/`** — CodeGraph symbol index. Absent → grep/Read still work, but `codegraph_explore` / `codegraph explore "…"` won't. Rebuild: `codegraph init` at the repo root (picked up live, no restart). Prefer it over grep once present (→ global CLAUDE.md § CodeGraph).
- **`.claude-task-master/`** — Task-master local state. Regenerated by the tool; nothing to restore.
- **App deps + services** — run `bin/setup` (prereqs → Rust/Bun deps → Docker Yugabyte+Dragonfly → migrate). Then `bin/check` must pass before you touch code.
- **Secrets** — `.env*` are git-ignored; never commit real secrets.

## Layout (target — see docs/architecture/01-monorepo-layout.md)
- `crates/*`   Rust libs: `protocol` `ecs-core` `sim` `netcode` `persistence` `cache` `scripting` `content-schema` `errors`
- `apps/*`     binaries: `gateway` `shard` `client` `mcp` `worldsvc` · `web` (Bun/Solid)
- `packages/*` TS shared for web: `i18n` `ui`
- `content/*`  data-driven game content (no recompile) + `manifest.json`
- `assets/*`   glTF / heightmaps / textures / audio (AI-generated, zstd)
- `docs/*`     `initial-idea/` (spec) · `architecture/` (design + `decisions/` ADRs)

## Where to look (load on demand)
- What & why (the spec)        → `docs/initial-idea/README.md`
- How it's built (design)      → `docs/architecture/README.md`
- Monorepo layout              → `docs/architecture/01-monorepo-layout.md`
- Netcode / sharding           → `docs/architecture/03-netcode-and-sharding.md`
- Anti-dupe / consistency      → `docs/architecture/04-data-and-consistency.md`
- Modding / ECS / scripting    → `docs/architecture/05-ecs-and-scripting.md`
- Operator website             → `docs/architecture/09-operator-web.md`
- Org standards                → `../gold-standards-in-ai/docs/`

## Planning
Non-trivial work → write a multi-file plan under `docs/plans/<YYYY>/<MM>/<DD>/<1NN>-<slug>/` (overview + slices + `status.yml`), then execute. → `../gold-standards-in-ai/docs/workflow/project-kickoff.md`

## Rules
SOLID/SRP. Tests always. Surgical diffs. Original IP only. Ownership → Yugabyte transaction, never cache.
