# 01 — Workflow & Parallelization

> **Big idea.** ~700 PRs, dozens of agents, one green `main`. This works only if agents **don't collide** and **don't block**. The mechanism: the [crate/app split](../../architecture/01-monorepo-layout.md) gives every track its own files; **published interfaces** let tracks integrate without reading each other's internals; and small, tested, auto-merged PRs keep `main` always shippable. → [master plan](README.md)

## The parallelization model
| Level | Unit | Owner | Isolation |
|---|---|---|---|
| **Track** | one big idea, ~100 fat PRs | one [subagent](../../../.claude/agents) | its own crates/apps |
| **Batch** | a milestone within a track = **one fat PR** | one agent run | a coherent set of files |
| **File** | one reason to change, ≤300 LOC | — | SRP, not a PR boundary |

Two agents never edit the same file in the same wave. If they would, the split is wrong — **fix the boundary, don't coordinate the edit**.

## <a name="contracts-first"></a>Contracts first
A downstream track opens as soon as the upstream **interface exists and compiles** — not when it's finished behind that interface.

- [`crates/protocol`](../../../crates/protocol) is the server↔client contract. Land the types (F4) and [server](30-game-server.md) + [client](40-game-client.md) build in parallel against them, each mocking the other side.
- [`crates/content-schema`](../../../crates/content-schema) is the core↔content contract. [Content](60-content-and-assets.md) authors data against the schema while [server](30-game-server.md) implements the loader.
- The **gateway/worldsvc HTTP contracts** are the server↔web contract. [Web](50-web-client.md) builds against the typed contract (Zod mirror of the Rust types) while [server](30-game-server.md) implements the endpoints.

**Changing a published contract is a cross-track event**: version it, announce it in the PR title (`contract(protocol): …`), and update every consumer in the same PR or a fast-follow.

## <a name="pr-conventions"></a>PR conventions — big fat PRs, machine-reviewed
A PR is a **whole batch**: a feature-complete, self-contained slice (many files, a working subsystem), not a micro-diff. ~100 such fat PRs per track. This scales because **review is automated** — [CodeRabbit](#merging--the-loop) + CI carry the load, so agents ship substantial units and the loop lands them.

- **One PR = one batch = one coherent capability.** It compiles, tests, and stands alone; it does not leave `main` half-built.
- **Fat PR, lean files.** The PR is large; each *file* still obeys SRP and ≤300 LOC. Big ≠ sprawling — it's a full subsystem split into small, single-purpose files.
- **Conventional commits**, scoped to the crate/app: `feat(shard): …`, `fix(persistence): …`, `contract(protocol): …`, `content(zones): …`, `docs(mvp): …`.
- **Tests always, in the same PR** — the tests that prove the batch ship with it. Rust: `cargo nextest`. Web: `bun test` + Playwright. A fat PR carries fat test coverage.
- **Green before merge** — `bin/check` (or `bun run --filter @omm/web test`) must pass. No red `main`, ever.
- **Self-reviewable** — PR body links the track + batch, lists what's in it, and states the Definition of Done it satisfies, so CodeRabbit and a skimming human both orient fast.

## <a name="merging--the-loop"></a>Merging — the loop
Big PRs land without a human bottleneck because the merge loop does the review-fix cycle. Merge with the [`/merge-pr` skill](../../../.claude/skills/merge-pr/SKILL.md) (wraps `claudetm merge-pr`): it monitors CI, fixes failures with the Claude SDK, addresses **CodeRabbit** review comments inline, resolves conflicts, and merges once everything is green — looping until mergeable. Agents open fat PRs; the loop reviews, fixes, and lands them. Humans spot-check only contract-change PRs. The bigger the PR, the more the automated loop earns its keep.

## <a name="documentation"></a>Documentation — a first-class deliverable
"Good docs for v1.0.0" is **built alongside the code, not after**. Three audiences, three homes:

| Audience | Home | Written by | v1.0.0 bar |
|---|---|---|---|
| **Developers / agents** | rustdoc on every public item + the [specs](../../specs/README.md) it implements | the track that ships the code | every public type/fn has a doc line; the spec's example compiles |
| **Operators** | `docs/operations/` — run a realm, config, scaling, backups | [server](30-game-server.md) + [web](50-web-client.md) + [db](10-database.md) | a stranger can stand up a realm from the docs alone |
| **Players / modders** | `docs/guide/` — play the slice; `content/README` + schema docs — author a datapack | [content](60-content-and-assets.md) + [client](40-game-client.md) | the modding walkthrough produces a loadable datapack |

**Rules:** every PR that adds a public API adds its rustdoc in the same PR (`missing_docs` is denied on library crates). Every track keeps its spec doc truthful — if the code diverges from the spec, the PR updates the spec. The [release gate](70-integration-and-release.md) fails if docs are stale. Concise English, reference-don't-repeat: link the spec, don't paraphrase it.

## Definition of Done (any unit)
Compiles, `bin/check` green, tests prove the behavior, public APIs documented, spec/plan links intact, merged via the loop. No orphaned TODOs, no `unwrap` on the hot path, no raw primitives across a boundary.

## Links
[master plan](README.md) · [foundation](00-foundation.md) · [layout](../../architecture/01-monorepo-layout.md) · [merge-pr skill](../../../.claude/skills/merge-pr/SKILL.md) · [CLAUDE.md](../../../CLAUDE.md)
