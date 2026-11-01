#!/usr/bin/env bun
/** bin/help — the DX catalog. What an agent (or human) runs, in one place. */
import { log } from './lib/log.ts';

log.step('Open-MMORPG — DX commands');
console.log(`
  bin/setup            fresh clone -> ready (deps, build, docker)
  bin/dev [target]     boot gateway|shard|worldsvc|client|web|all
  bin/check            the gate: rust (fmt/clippy/tests --all-features) + content validation + web (lint/typecheck/test/build)
  bin/fmt              auto-format (cargo fmt + biome)
  bin/new-module NAME  scaffold a compiled gameplay module under modules/ (doc 10)

  Web-specific:
  bin/test-web         run web unit tests
  bin/test-e2e         run web E2E tests (Playwright)
  bin/check-web        web quality gates: typecheck + lint + unit tests

  Direct (Bun TS, no shim):
  bun scripts/check.ts --rust-only | --web-only
  bun scripts/dev.ts web

  Under the hood:
  cargo fmt --all --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo nextest run --workspace --all-features (or cargo test --workspace --all-features)
  bun scripts/validate-content.ts
  cargo bench --package omm-content-schema --no-run
  bun run lint && bun run typecheck && bun test && bun run --filter @omm/web build
`);
log.dim('  Scripts live in scripts/*.ts; reusable logic in scripts/lib/*.ts.');
