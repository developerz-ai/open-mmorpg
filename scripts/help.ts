#!/usr/bin/env bun
/** bin/help — the DX catalog. What an agent (or human) runs, in one place. */
import { log } from './lib/log.ts';

log.step('Open-MMORPG — DX commands');
console.log(`
  bin/setup            fresh clone -> ready (deps, build, docker)
  bin/dev [target]     boot gateway|shard|worldsvc|client|web|all
  bin/check            the gate: fmt + clippy -D warnings + tests (+ web)
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
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --workspace --all-features
  bun run lint && bun run typecheck && bun test
`);
log.dim('  Scripts live in scripts/*.ts; reusable logic in scripts/lib/*.ts.');
