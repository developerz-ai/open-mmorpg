#!/usr/bin/env bun
/** bin/fmt — auto-format everything: cargo fmt + biome --write. */
import { runInherit } from './lib/cmd.ts';
import { log } from './lib/log.ts';

log.step('rust: cargo fmt');
runInherit(['cargo', 'fmt', '--all']);

log.step('web: biome --write');
runInherit(['bun', 'run', 'lint:fix']);

log.ok('formatted');
