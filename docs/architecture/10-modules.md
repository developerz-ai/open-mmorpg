# 10 — Compiled Modules (fork without merge conflicts)

> AzerothCore-style modules, in pure Cargo. A fork adds or changes a server feature by dropping a **self-contained Rust crate** under `modules/` that implements the core's hook traits — **no core file is edited**. Upstream updates then pull cleanly, because the fork never touched `crates/` or `apps/`.

## Why compiled, not `.so` plugins
The Rust ABI is unstable — runtime shared-object plugins are fragile across compiler versions and break silently. So, like AzerothCore itself, we **compile modules in**: each is an ordinary workspace crate linked at build time. The trade is a rebuild when a module changes; incremental compilation and the shared `rust-cache` keep that cheap (see [DX](#dx-fast-cached-builds)).

## This vs. content scripting ([05](05-ecs-and-scripting.md))
Two different extensibility seams — pick by what you're changing:

| | **Content** ([05](05-ecs-and-scripting.md)) | **Modules** (this doc) |
|---|---|---|
| Changes | factions, classes, quests, ability *data* | server *behaviour* / new gameplay systems |
| Form | data + sandboxed WASM/Lua under `content/` | compiled Rust crate under `modules/` |
| Recompile? | no | yes (linked in) |
| Trust | untrusted, fuel-metered sandbox | first-party / fork-authored, full native |
| Reach | narrow capability API | the core hook traits |

If it's a number or a definition, it's content. If it's new engine behaviour reacting to server events, it's a module.

## The three pieces
```
crates/module-api   the hook traits + ModuleHost + declare_module!  (a leaf crate)
crates/modules      generated aggregator: force-links + builds a populated host
modules/<name>/     one crate per module: module.toml + src/lib.rs
```

### 1. `module-api` — the contract
- **`ServerHooks`** — one method per server event, each a default no-op, so a module implements only what it cares about and adding a new hook never breaks existing modules. Hooks take `&self` (a module is a long-lived, shared observer) and must be non-blocking — they run on the authoritative tick path.
- **`*Ctx`** payloads — immutable event data in typed ids (`AccountId`, `EntityId`, …), never raw `u64`.
- **`Module`** — a `ServerHooks` impl + a `ModuleManifest` identity, object-safe and downcastable.
- **`ModuleHost`** — holds every loaded module and *is itself* a `ServerHooks`: the core calls one surface, the host fans out to each module in deterministic order.
- **`declare_module!`** — emits the `module()` entry point the registry links against.

### 2. `modules` — the generated registry (the "no core edits" trick)
Cargo can't glob *dependencies*, and the Rust linker drops crates nothing references — so a module is linked in only if something names it. `crates/modules` is that something:
- Its `build.rs` scans `../../modules/*`, and generates `register_all(&mut ModuleHost)` — one `host.register(<crate>::module())` per module, in **sorted directory order** (deterministic dispatch, which the anti-cheat re-sim relies on).
- Its `Cargo.toml` carries a `path` dependency per module. That dep is what actually force-links the crate; `bin/new-module` appends the one line. This is the *only* per-module manifest edit, and it lives in the module system's own glue — never in core gameplay files, so upstream merges stay clean.
- `omm_modules::load()` runs `register_all` into a fresh host. An empty `modules/` yields an empty host: a valid no-op, not an error.

### 3. `modules/<name>/` — a module
```
modules/hello-world/
  module.toml     name, version, core-api-version, declared hooks
  src/lib.rs      impl ServerHooks + impl Module + declare_module!(…)
```
`module.toml` is the build-time manifest (humans + tooling read it; `build.rs` re-runs discovery when it changes). `ModuleManifest` mirrors its identity at runtime for the boot log and operator tooling.

## Auto-discovery, end to end
```
modules/*  ─ workspace glob ─▶  compiled + nextest-tested   (Cargo.toml `members`)
modules/*  ─ build.rs scan  ─▶  register_all()              (force-link + dispatch)
```
The workspace `members = ["modules/*"]` glob means **every** module builds and its tests run with zero config. The generated `register_all` links them into the server. Two independent mechanisms, both keyed off the same directory.

## The hooks
| Hook | Fires on | Status |
|---|---|---|
| `on_tick` | every fixed sim step, after `World::step` | **live** — wired in `apps/shard` `main.rs` |
| `on_player_login` | the authoritative accept handshake spawns a player | **API-complete** — fires where the accept path lands (session-accept slice); dispatch tested via the host today |
| `on_creature_death` | an actor dies | **API-complete** — awaits a sim death event |
| `on_loot` | an item is looted | **API-complete** — awaits a sim loot system |
| `on_chat` | a player sends a chat message | **API-complete** — awaits chat wiring; `worldsvc` social layer is the likely source |
| `on_level_up` | a player's level advances | **API-complete** — awaits sim XP/level-up system |
| `on_zone_enter` | a player crosses a zone boundary | **API-complete** — awaits sim zone-transition system |
| `on_item_use` | a player activates an inventory item | **API-complete** — awaits sim item-use system |

"API-complete" = the hook, its context, and host dispatch are defined and tested; the single core call site is added when the event source exists. Wiring one is a one-line `host.on_x(&ctx)` at the event, never a change to the module system.

The proof-of-concept `modules/hello-world` implements `on_tick` and `on_player_login`, counts each, and logs. The full example `modules/milestones` implements `on_level_up`, `on_zone_enter`, `on_item_use`, `on_chat`, and `on_tick` to unlock and track named milestones end to end; `crates/modules/tests/generated_registry.rs` proves both modules are discovered, force-linked, and receive events dispatched through the real host.

## Adding a module
```
bin/new-module fast-travel      # scaffolds modules/fast-travel/ + registers the dep
# edit modules/fast-travel/src/lib.rs — implement the hooks you need
bin/check                       # compiles it in, runs its tests
```
No core file is touched. That is the whole point.

## <a name="dx-fast-cached-builds"></a>DX: fast, cached builds
- Modules are workspace crates → **incremental compilation** rebuilds only the changed module, and the existing CI `rust-cache` caches their artifacts across runs.
- Tests run under `cargo nextest` in parallel with the rest of the workspace — a module's tests are just more crate tests.
- Optional local speedup: install `sccache` and set `RUSTC_WRAPPER=sccache` (or add it under `[build]` in a git-ignored `.cargo/config.toml`) to share a compilation cache across branches. Not required; incremental + `rust-cache` already keep `bin/check` fast.

## Server-first, then engine and client
The pattern generalizes: a trait of default-noop hooks + a generated registry that force-links crates from a conventional directory. The engine and client adopt the same shape next — an `EngineHooks` (asset/scene lifecycle) and `ClientHooks` (input/HUD/prediction) trait, each with its own `modules/` discovery — so a fork extends any layer the same way. Only the trait surface and the directory differ; the discovery machinery is identical.

## Invariants
- **Deterministic dispatch.** Modules run in sorted-directory order; same events → same sequence on every box.
- **Non-blocking hooks.** Bodies run on the tick path; a stall or panic there drops every player on the shard. Keep them cheap.
- **Additive API.** New hooks get default-noop methods — every existing module continues to compile and behave identically when a hook is added. The `core-api-version` in `module.toml` bumps *only* on a breaking change (signature change, removal, or semantic shift); an additive hook addition never increments it. Because modules compile in-workspace, an incompatible one fails at *compile* time — never the silent stale-plugin failure `.so`s invite.
