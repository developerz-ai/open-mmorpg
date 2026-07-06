import { describe, expect, test } from 'bun:test';
import {
  cargoToml,
  DEPS_MARKER,
  dependencyLine,
  deriveNames,
  InvalidModuleName,
  insertDependency,
  libRs,
  moduleToml,
} from './module-scaffold.ts';

describe('deriveNames', () => {
  test('derives crate, ident, and type from kebab-case', () => {
    const n = deriveNames('fast-travel');
    expect(n).toEqual({
      name: 'fast-travel',
      crate: 'omm-module-fast-travel',
      ident: 'omm_module_fast_travel',
      type: 'FastTravel',
    });
  });

  test('handles a single-word name', () => {
    expect(deriveNames('economy').type).toBe('Economy');
    expect(deriveNames('economy').ident).toBe('omm_module_economy');
  });

  test('trims surrounding whitespace', () => {
    expect(deriveNames('  loot-tables \n').name).toBe('loot-tables');
  });

  test.each([
    'Fast-Travel', // uppercase
    'fast_travel', // underscore
    '-leading',
    'trailing-',
    'double--hyphen',
    '1numeric-start',
    '',
    'has space',
  ])('rejects invalid name %p', (bad) => {
    expect(() => deriveNames(bad)).toThrow(InvalidModuleName);
  });
});

describe('templates', () => {
  const n = deriveNames('fast-travel');

  test('Cargo.toml names the crate and lib', () => {
    const toml = cargoToml(n);
    expect(toml).toContain('name = "omm-module-fast-travel"');
    expect(toml).toContain('name = "omm_module_fast_travel"');
    expect(toml).toContain('omm-module-api.workspace = true');
  });

  test('module.toml carries the manifest fields', () => {
    const toml = moduleToml(n);
    expect(toml).toContain('name = "fast-travel"');
    expect(toml).toContain('core-api-version = 1');
  });

  test('lib.rs defines the module type and entry point', () => {
    const rs = libRs(n);
    expect(rs).toContain('pub struct FastTravel');
    expect(rs).toContain('impl Module for FastTravel');
    expect(rs).toContain('declare_module!(FastTravel::default());');
    expect(rs).toContain('ModuleManifest::new("fast-travel"');
  });
});

describe('insertDependency', () => {
  const n = deriveNames('fast-travel');
  const base = `[dependencies]\nomm-module-api.workspace = true\n\n${DEPS_MARKER}\nomm-module-hello-world = { path = "../../modules/hello-world" }\n`;

  test('inserts the dep line right after the marker', () => {
    const out = insertDependency(base, n);
    expect(out).toContain(dependencyLine(n));
    const lines = out.split('\n');
    const markerIdx = lines.indexOf(DEPS_MARKER);
    expect(lines[markerIdx + 1]).toBe(dependencyLine(n));
  });

  test('is idempotent', () => {
    const once = insertDependency(base, n);
    expect(insertDependency(once, n)).toBe(once);
  });

  test('throws when the marker is missing', () => {
    expect(() => insertDependency('[dependencies]\n', n)).toThrow(/marker not found/);
  });
});
