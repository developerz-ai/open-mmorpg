#!/usr/bin/env bun
/**
 * bin/setup — fresh clone to ready. Checks prereqs, installs JS deps, warms the
 * Rust build, and (best-effort) boots the local data stores via Docker.
 */
import { have, runInherit } from './lib/cmd.ts';
import { log } from './lib/log.ts';
import { fromRoot } from './lib/paths.ts';

log.step('Checking prerequisites');
for (const bin of ['cargo', 'bun']) {
  if (!have(bin)) {
    log.fail(`${bin} not found — install it and re-run bin/setup`);
    process.exit(1);
  }
}
log.ok('cargo + bun present');

log.step('Installing JS deps');
if (runInherit(['bun', 'install']) !== 0) {
  log.fail('bun install failed');
  process.exit(1);
}
log.ok('deps installed');

log.step('Warming the Rust build');
runInherit(['cargo', 'build', '--workspace']);

if (have('docker')) {
  log.step('Starting data stores (Yugabyte + Dragonfly)');
  const code = runInherit([
    'docker',
    'compose',
    '-f',
    fromRoot('docker/docker-compose.yml'),
    'up',
    '-d',
  ]);
  if (code === 0) log.ok('data stores up');
  else log.warn('docker compose failed — start services manually if needed');
} else {
  log.warn('docker not found — skipping data stores');
}

log.ok('Setup complete. Next: bin/dev');
