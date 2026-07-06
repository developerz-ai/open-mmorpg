#!/usr/bin/env bun
/**
 * bin/setup — fresh clone to ready. Checks prereqs, installs JS deps, warms the
 * Rust build, and (best-effort) boots the local data stores via Docker.
 */
import { have, run, runInherit } from './lib/cmd.ts';
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

// Check for Linux build dependencies needed for Bevy
if (process.platform === 'linux') {
  const linuxDeps = ['libasound2-dev', 'libudev-dev', 'libx11-dev', 'pkg-config', 'g++'];
  const missing = linuxDeps.filter((dep) => {
    // `dpkg -l` reports success for `rc` (removed, config-files-left) packages;
    // require the explicit "install ok installed" state instead.
    const res = run(['dpkg-query', '-s', dep]);
    return res.code !== 0 || !res.stdout.includes('install ok installed');
  });
  if (missing.length > 0) {
    log.warn(
      `Missing Linux development libraries. Install with:\n  sudo apt-get install -y ${missing.join(' ')}`,
    );
  }
}

log.step('Installing JS deps');
if (runInherit(['bun', 'install']) !== 0) {
  log.fail('bun install failed');
  process.exit(1);
}
log.ok('deps installed');

log.step('Warming the Rust build');
runInherit(['cargo', 'build', '--workspace']);

log.step('Installing Playwright browsers for E2E tests');
const playwrightInstall = runInherit([
  'bun',
  'run',
  '--filter',
  '@omm/web',
  '--silent',
  'exec',
  'playwright',
  'install',
  'chromium',
]);
if (playwrightInstall === 0) log.ok('Playwright browsers installed');
else log.warn('Playwright browser install failed — E2E tests may not work');

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
