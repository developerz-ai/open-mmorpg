import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

describe('Progress component exports', () => {
  test('Progress.tsx exists and exports Progress', () => {
    const progressPath = join(import.meta.dir, 'Progress.tsx');
    const content = readFileSync(progressPath, 'utf-8');
    expect(content).toContain('export function Progress');
    expect(content).toContain('export interface ProgressProps');
  });

  test('index.ts exports Progress', () => {
    const indexPath = join(import.meta.dir, 'index.ts');
    const content = readFileSync(indexPath, 'utf-8');
    expect(content).toContain('Progress');
    expect(content).toContain('ProgressProps');
  });
});

describe('Progress component structure', () => {
  test('Progress has ARIA attributes for accessibility', () => {
    const progressPath = join(import.meta.dir, 'Progress.tsx');
    const content = readFileSync(progressPath, 'utf-8');
    expect(content).toContain('role="progressbar"');
    expect(content).toContain('aria-valuenow');
    expect(content).toContain('aria-valuemin');
    expect(content).toContain('aria-valuemax');
    expect(content).toContain('aria-label');
  });

  test('Progress accepts value prop (0-100)', () => {
    const progressPath = join(import.meta.dir, 'Progress.tsx');
    const content = readFileSync(progressPath, 'utf-8');
    expect(content).toContain('value?: number');
    expect(content).toContain('Math.min(100, Math.max(0');
  });

  test('Progress requires label prop', () => {
    const progressPath = join(import.meta.dir, 'Progress.tsx');
    const content = readFileSync(progressPath, 'utf-8');
    expect(content).toContain('label: string');
  });
});

describe('Progress component styles', () => {
  test('Progress has container styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.omm-progress');
    expect(content).toContain('border-radius: var(--radius-pill)');
  });

  test('Progress has bar styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.omm-progress__bar');
    expect(content).toContain('transition: width');
  });

  test('Progress bar uses accent color', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('background: rgb(var(--color-accent))');
  });
});

describe('Spinner size variants', () => {
  test('Spinner.tsx exports SpinnerSize type', () => {
    const spinnerPath = join(import.meta.dir, 'Spinner.tsx');
    const content = readFileSync(spinnerPath, 'utf-8');
    expect(content).toContain('export type SpinnerSize');
  });

  test('Spinner.tsx has size prop', () => {
    const spinnerPath = join(import.meta.dir, 'Spinner.tsx');
    const content = readFileSync(spinnerPath, 'utf-8');
    expect(content).toContain('size?: SpinnerSize');
  });

  test('index.ts exports SpinnerSize', () => {
    const indexPath = join(import.meta.dir, 'index.ts');
    const content = readFileSync(indexPath, 'utf-8');
    expect(content).toContain('SpinnerSize');
  });

  test('CSS has all size variants', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.spinner--sm');
    expect(content).toContain('.spinner--md');
    expect(content).toContain('.spinner--lg');
  });

  test('Size variants have distinct sizes', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    // sm should be smaller than md (default was 1.1rem)
    expect(content).toContain('.spinner--sm');
    expect(content).toContain('width: 0.8rem');
    // lg should be larger than md
    expect(content).toContain('.spinner--lg');
    expect(content).toContain('width: 1.6rem');
  });
});
