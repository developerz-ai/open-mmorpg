# Contributing to Open-MMORPG

Thanks for helping build the best open-core MMORPG engine. This repo is engineered so an AI coding agent ships production-quality work from the first prompt — the same rules keep human contributions sharp. Read this once; it's short.

## Get started

```
bin/setup            # prereqs → deps → docker (Yugabyte+Dragonfly) → migrate
bin/dev [target]     # boot gateway/shard/client/web with reload
bin/check            # the gate: fmt-check + clippy -D warnings + nextest (+ biome/tsc for web)
bin/fmt              # cargo fmt + biome --write
```

`bin/check` is **the gate** — local, git hook, and CI all run the same command. If it's green, your change is mergeable; if it's red, it isn't.

Scripts live in `scripts/*.ts` (Bun TypeScript); the `bin/*` entries are thin shims that call them. Edit the `.ts`, not the shim.

## Conventions — read before non-trivial work

The contract every contributor (human or agent) reads first is **[CLAUDE.md](CLAUDE.md)**, backed by **[docs/](docs/)** (`initial-idea/` = what & why, `architecture/` = how). Don't re-litigate the binding decisions listed there. Highlights:

**Rust (`crates/`, `apps/*` binaries):**
- **No `unwrap`/`expect` outside `main`/tests** — a hot-path panic drops every player on the shard.
- **Typed errors** (`thiserror`), stable client codes; never leak credentials or internal detail in errors/logs.
- Newtypes over primitives (`AccountId`, `ItemId`), borrow by default (`&str`/`&[T]`), async end-to-end (`tokio::sync`).
- **Files ≤300 LOC. One crate = one reason to change.** SOLID/SRP.
- Make illegal states unrepresentable — the type system is the first anti-cheat layer.

**Web (`apps/web`, Bun + SolidJS):**
- TypeScript strict (no `any`), **Zod at every boundary**, custom typed errors.
- Every user-facing string via `t()` — missing keys render loudly (`⟦key⟧`).
- **Dark theme only.** Semantic CSS-variable tokens (`bg-bg text-fg`), never raw hex.
- SRP — one component, one job, ≤300 LOC; shared UI → `packages/ui`.

**Everywhere:** surgical diffs (touch only what the task requires, match existing style), and **tests always** (bug → reproducing test → make it pass).

## The extensibility line

Core is compiled (`crates/`), content is data (`content/`). If changing a faction, class, or quest needs `cargo build`, you're editing the wrong layer. Prefer data.

## Pull requests

1. Branch off `main`.
2. Keep it small and focused — one concern per PR.
3. Keep `bin/check` green.
4. Conventional-commit-ish titles (`feat:`, `fix:`, `docs:`, `refactor:`…).
5. Fill in the PR template. Architectural changes need a docs update or an ADR under `docs/architecture/decisions/`.

Every PR gets an **automated AI review**, then **human review**, before merge. No single layer is trusted to be perfect.

## Built by AI agents

Agents write the code; humans review, steer, and operate. Contributions from either are welcome and held to the same gate.

## Original IP only

Never extract, rip, or port any real game's asset, name, or data table. Everything here is 100% original, MIT-safe to commercialize — keep it that way. → [docs/initial-idea/01-legal-and-licensing.md](docs/initial-idea/01-legal-and-licensing.md)

## Certify your contribution

This project is [MIT licensed](LICENSE). By opening a pull request you certify that you wrote the contribution yourself (or otherwise have the right to submit it) and that it is your original work, and you agree to license it under the MIT License. A simple `Signed-off-by: Your Name <you@example.com>` line in your commit (`git commit -s`) is a welcome way to make that explicit.

## Conduct

Be decent. See the [Code of Conduct](CODE_OF_CONDUCT.md).
