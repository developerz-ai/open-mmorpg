import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

describe('Select component exports', () => {
  test('Select.tsx exists and contains Select export', () => {
    const selectPath = join(import.meta.dir, 'Select.tsx');
    const content = readFileSync(selectPath, 'utf-8');
    expect(content).toContain('export function Select');
    expect(content).toContain('export function SelectOption');
  });

  test('index.ts exports Select components', () => {
    const indexPath = join(import.meta.dir, 'index.ts');
    const content = readFileSync(indexPath, 'utf-8');
    expect(content).toContain('Select');
    expect(content).toContain('SelectOption');
    expect(content).toContain('SelectProps');
    expect(content).toContain('SelectOptionProps');
  });

  test('Select has keyboard navigation features', () => {
    const selectPath = join(import.meta.dir, 'Select.tsx');
    const content = readFileSync(selectPath, 'utf-8');
    expect(content).toContain('ArrowDown');
    expect(content).toContain('ArrowUp');
    expect(content).toContain('Enter');
    expect(content).toContain('Escape');
    expect(content).toContain('role="combobox"');
    expect(content).toContain('role="option"');
  });

  test('Select has a11y attributes', () => {
    const selectPath = join(import.meta.dir, 'Select.tsx');
    const content = readFileSync(selectPath, 'utf-8');
    expect(content).toContain('aria-expanded');
    expect(content).toContain('aria-selected');
    expect(content).toContain('aria-invalid');
    expect(content).toContain('tabIndex');
  });
});
