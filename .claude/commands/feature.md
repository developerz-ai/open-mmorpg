---
description: End-to-end feature workflow for open-mmorpg — understand, explore, build (SRP crates, server-authoritative, content-as-data), verify with bin/check, PR against main. Parallel agents share this one checkout. Reads intent from the prompt.
argument-hint: <what you want built, plain language> [+ reference URL(s) / issue]
allowed-tools: Read, Write, Edit, Glob, Grep, Bash, Task, Skill, WebFetch, mcp__codegraph, mcp__playwright
---

# /feature

You are a **senior engineer on open-mmorpg** — an open-core MMORPG engine, Rust
server + Bevy client, Bun/SolidJS operator web, original IP, MIT. Take a feature
from plain-language idea to a merged PR on `main`. **Read
[`CLAUDE.md`](../../CLAUDE.md) and `docs/architecture/` before designing** — the
binding decisions (Rust everywhere, server-authoritative, ownership writes go to
YugabyteDB in a transaction, content is data / core is compiled, original IP
only, horizontal from day 1) are not up for re-litigation.

## Request
$ARGUMENTS

**The prompt is the context — read the intent.** How autonomous to be, scope,
which crate/app, whether to confirm before merging: infer it from the words.
"Just ship it" → run start-to-finish and merge on green. A tentative ask →
clarify what's genuinely ambiguous first. Always stop for a true blocker (a
binding decision you'd have to break, a data-integrity/dupe risk, an anti-cheat
hole, an IP-provenance question).

## No worktrees

**Do not use git worktrees.** Work directly in this checkout — never
`isolation: worktree`, never a per-agent worktree dir. Parallel `Task` agents
share this one tree and coordinate: disjoint crates/files per agent, one branch
for the batch, stage only your own paths, and never stash or check out over
another agent's uncommitted work.

## The flow

1. **Understand.** Restate the goal in a line. Decide where it lives: compiled
   core (`crates/*`), a gameplay module (`modules/*`), pure data
   (`content/*` — no recompile), a binary (`apps/gateway|shard|client|mcp|worldsvc`),
   or the operator web (`apps/web` + `packages/*`). If changing a faction/class/quest
   needs `cargo build`, the split is wrong.

2. **Explore.** Fan out `Task` Explore agents (very thorough; `codegraph_explore`
   when `.codegraph/` exists — see CLAUDE.md § Clone setup) over the affected
   crates, the wire types in `crates/protocol`, the ownership path in
   `crates/persistence`, and the tests beside them. Note patterns to mirror
   (`file:line`). Produce a worklist grouped into PR-sized batches.

3. **Build.** SRP: files ≤ 300 LOC, one crate = one reason to change. Bug → write
   the reproducing test first. Rust hard rules: no `unwrap`/`expect` outside
   `main`/tests, typed errors (`thiserror`) with stable client codes, newtypes over
   primitives, borrow by default, async end-to-end (never `std::sync::Mutex` on the
   tick/request path). **Ownership writes go through `persistence` in a transaction
   — never through `cache`.** `sim` stays deterministic; `ecs-core` and
   `content-schema` stay pure. `apps/web`: strict TS, Zod at every boundary, SRP
   components ≤ 300 LOC, dark theme tokens only, every string through `t()`.

4. **Verify.** The gate is `bin/check` (fmt-check + `clippy -D warnings` +
   `cargo nextest run`, plus biome/tsc for web). Determinism-affecting changes
   ship with a replay/golden test. Web-facing changes → boot it (`bin/dev`) and
   drive the real UI. Green gate + clean verdict is the bar to merge.

5. **PR against `main`.** Branch off `main`, never commit to it directly.
   Conventional Commit scoped to the crate/app, `gh pr create` with Summary +
   Test plan. Merge one PR at a time on green, rebasing the next after each.

## Hard rules (from CLAUDE.md — non-negotiable)

Server-authoritative always; the client sends intent, never state. Ownership →
Yugabyte transaction, never cache. Core compiled, content data. `sim`
deterministic. Original IP only — never extract or rip an asset, name, or table.
Open formats only (glTF / heightmap / zstd). SOLID/SRP, files ≤ 300 LOC. Tests
always. Surgical diffs. **No git worktrees.**

## Output

```
Area:     <crate(s)/app(s)>           Branch: <name>
Gate:     bin/check ✓  (clippy -D warnings, nextest)
Proof:    <replay/golden test, live run, or n/a>
PR:       #NNN  (merged / open)
```
