import { resolve } from 'node:path';

/** Absolute path to the repo root, derived from this file's location. */
export const repoRoot = resolve(import.meta.dir, '..', '..');

/** Resolve a path relative to the repo root. */
export function fromRoot(...parts: string[]): string {
  return resolve(repoRoot, ...parts);
}
