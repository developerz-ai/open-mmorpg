#!/usr/bin/env bun
/**
 * bin/new-module <name> — scaffold a compiled gameplay module.
 *
 * Creates `modules/<name>/` (Cargo.toml, module.toml, src/lib.rs) from a
 * template and registers its dependency in `crates/modules/Cargo.toml`. After
 * this the module is auto-discovered, compiled, and dispatched — no core file is
 * edited. See docs/architecture/10-modules.md.
 */
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { parseArgs } from './lib/args.ts';
import { log } from './lib/log.ts';
import {
  cargoToml,
  deriveNames,
  InvalidModuleName,
  insertDependency,
  libRs,
  moduleToml,
} from './lib/module-scaffold.ts';
import { fromRoot } from './lib/paths.ts';

const { positionals } = parseArgs(Bun.argv.slice(2));
const raw = positionals[0];

if (raw === undefined) {
  log.fail('usage: bin/new-module <name>   (lowercase kebab-case, e.g. fast-travel)');
  process.exit(2);
}

let names: ReturnType<typeof deriveNames>;
try {
  names = deriveNames(raw);
} catch (err) {
  if (err instanceof InvalidModuleName) {
    log.fail(err.message);
    process.exit(2);
  }
  throw err;
}

const dir = fromRoot('modules', names.name);
if (existsSync(dir)) {
  log.fail(`module already exists: modules/${names.name}`);
  process.exit(1);
}

// Write the module crate.
mkdirSync(fromRoot('modules', names.name, 'src'), { recursive: true });
writeFileSync(fromRoot('modules', names.name, 'Cargo.toml'), cargoToml(names));
writeFileSync(fromRoot('modules', names.name, 'module.toml'), moduleToml(names));
writeFileSync(fromRoot('modules', names.name, 'src', 'lib.rs'), libRs(names));
log.ok(`created modules/${names.name}/`);

// Register its dependency in the aggregator so it links in.
const aggPath = fromRoot('crates', 'modules', 'Cargo.toml');
const updated = insertDependency(readFileSync(aggPath, 'utf8'), names);
writeFileSync(aggPath, updated);
log.ok(`registered ${names.crate} in crates/modules/Cargo.toml`);

log.step('next steps');
log.dim(`  1. edit modules/${names.name}/src/lib.rs — implement the hooks you need`);
log.dim(`  2. bin/check   # compiles the module in and runs its tests`);
