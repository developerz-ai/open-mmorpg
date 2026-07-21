---
description: Write a concise, self-contained execution plan to docs/plans/<YYYY>/<MM>/<DD>/<1NN>-<slug>/ for another AI to implement
argument-hint: [what you want done]
allowed-tools: Write, Read, Glob, Grep, Task, Bash
---

# /planx

Produce a concise plan another AI can execute with zero extra context. Plan only
— no implementation, no code execution, no edits outside the plan dir.

## Goal
$ARGUMENTS

## Steps

1. **Resolve path.** Run `date +%Y`, `date +%m`, `date +%d`. Dir =
   `docs/plans/<YYYY>/<MM>/<DD>/`. `Glob docs/plans/<YYYY>/<MM>/<DD>/1*` → next
   number = highest existing `1NN-*` + 1, else `101`. Slug = kebab-case title,
   max 5 words. Final plan dir: `docs/plans/<YYYY>/<MM>/<DD>/<1NN>-<slug>/`.

2. **Explore.** `Task` (subagent_type=Explore, thoroughness="very thorough"):
   existing patterns and files to touch (`file:line`); which `crates/*`,
   `modules/*`, `apps/*`, `packages/*` or `content/*` are affected; wire types in
   `crates/protocol`; the ownership path through `crates/persistence`; purity
   constraints on `ecs-core` / `content-schema` / `sim`; tests beside the code;
   gotchas. Prefer `codegraph_explore` for structural lookups when `.codegraph/`
   exists. Skip only for trivial asks.

3. **Write the plan as multiple files** — never one big `plan.md`. Always an
   `overview.md` index plus one `<NN>-<aspect>.md` per separable area (e.g.
   `01-protocol.md`, `02-sim.md`, `03-persistence.md`, `04-content-schema.md`,
   `05-web.md`, `06-tests.md`).

   **`overview.md`** — Goal (1–2 sentences, what + why); Context (stack facts:
   Rust + tokio, Bevy client, YugabyteDB durable / Dragonfly ephemeral, Bun +
   SolidJS operator web — only what's relevant) + reference patterns as
   `file:line`; Plan files in execution order with a one-line summary each; Done
   when (verifiable acceptance criteria); Risks / open questions.

   **Each `<NN>-<aspect>.md`** — a header line pointing back at `overview.md` and
   naming its dependency; Files to change (`path:line` — what, why); Steps
   (ordered, concrete, referencing `file:line` / `Type::method` rather than
   restating them); Tests (what to add/run; gate is `bin/check` —
   `cargo nextest run`, `cargo clippy --all-targets -D warnings`,
   `bun run --filter @omm/web test`); Done when.

4. **Write a `status.yml`** in the plan dir — the live tracker. New plans start
   `not_started` / `0%`. `created_by` + `owner` from `git config user.name`;
   leave `worked_by: ""` (the executor fills it with their own git user.name).
   Keys: `plan`, `title`, `status` (not_started | in_progress | blocked |
   complete | superseded), `created_by`, `worked_by`, `owner`, `percent`,
   `current_focus`, `slices` (one row per slice: `file`, `status`, `percent`),
   `evidence` (commits/PRs), `notes`, `last_updated`. Valid YAML, those enums.
   It is the one tracker in the dir — the `.md` slices stay reference maps.

## Rules

- Compact English. Fragments over sentences. `file:line` and `Type::method` refs
  over prose. Tables for structured data.
- Reference-only: point at code, don't paste it or re-explain it.
- No checkboxes. Plain bullets. The slices are a reference map, not a tracker.
- Multiple files always: `overview.md` + `<NN>-<aspect>.md`. Never one `plan.md`.
- Self-contained: the executor reads only `overview.md`, its slice, and the files
  those cite.
- **No git worktrees** — plans execute directly in this checkout. Never plan for
  `isolation: worktree` or per-agent worktree dirs; parallel agents share this
  tree and take disjoint crates/files.
- Respect the binding decisions in `CLAUDE.md`: server-authoritative; ownership
  writes → Yugabyte transaction, never cache; core compiled / content data;
  `sim` deterministic; `ecs-core` + `content-schema` pure; original IP only; open
  formats only. Rust rules: no `unwrap`/`expect` outside `main`/tests, typed
  errors, newtypes, async end-to-end, files ≤ 300 LOC.

## Output
```
✓ docs/plans/<YYYY>/<MM>/<DD>/<1NN>-<slug>/overview.md
  + 01-<aspect>.md, 02-<aspect>.md, … (one per area)
  + status.yml (tracker — status/owner/percent/current_focus)
Next: run an executor on overview.md.
```
