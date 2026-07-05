## Summary

<!-- One or two sentences. What does this PR do? -->

## What & why

<!-- The change and the reasoning. Link the issue/ADR/doc it traces to. -->

## How verified

<!-- Show it's green and correct. -->

- [ ] `bin/check` passes locally (fmt-check + clippy -D warnings + nextest + biome/tsc)
- [ ] Tests added/updated and passing
- [ ] Screenshots / recording attached for web (`apps/web`) changes

## Checklist

- [ ] **Surgical diff** — touches only what the task requires; matches existing style
- [ ] **Tests** — added or updated (bug → reproducing test first)
- [ ] **Docs / ADR** — updated for any architectural change (`docs/architecture/decisions/`)
- [ ] **No `unwrap`/`expect`** on hot paths (outside `main`/tests); typed errors
- [ ] **Original IP only** — no extracted/ripped asset, name, or data table
- [ ] **i18n** — every user-facing web string goes through `t()` (no hardcoded copy)
- [ ] **Extensibility line** — content changes are data (`content/`), not compiled (`crates/`)
