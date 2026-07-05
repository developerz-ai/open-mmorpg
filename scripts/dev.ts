#!/usr/bin/env bun
/**
 * bin/dev [target] — boot what you're working on with reload.
 * Targets: gateway | shard | worldsvc | client | web | all (default).
 */
import { parseArgs } from './lib/args.ts';
import { runInherit } from './lib/cmd.ts';
import { log } from './lib/log.ts';

const TARGETS: Record<string, string[]> = {
  gateway: ['cargo', 'run', '-p', 'omm-gateway'],
  shard: ['cargo', 'run', '-p', 'omm-shard'],
  worldsvc: ['cargo', 'run', '-p', 'omm-worldsvc'],
  client: ['cargo', 'run', '-p', 'omm-client'],
  web: ['bun', 'run', '--filter', '@omm/web', 'dev'],
};

const target = parseArgs(Bun.argv.slice(2)).positionals[0] ?? 'all';

if (target === 'all') {
  // The server base + the operator site, together, for a full local loop.
  log.step('booting gateway + shard + web (Ctrl-C to stop)');
  const procs = [TARGETS.gateway, TARGETS.shard, TARGETS.web].map((cmd) =>
    Bun.spawn(cmd as string[], { stdout: 'inherit', stderr: 'inherit' }),
  );
  process.on('SIGINT', () => {
    for (const p of procs) p.kill();
    process.exit(0);
  });
  await Promise.all(procs.map((p) => p.exited));
} else if (target in TARGETS) {
  const cmd = TARGETS[target];
  if (cmd) process.exit(runInherit(cmd));
} else {
  log.fail(`unknown target '${target}'. Options: ${Object.keys(TARGETS).join(', ')}, all`);
  process.exit(2);
}
