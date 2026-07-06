import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

describe('Tooltip component exports', () => {
  test('Tooltip.tsx exists and contains Tooltip exports', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('export function Tooltip');
    expect(content).toContain('export interface TooltipProps');
  });

  test('index.ts exports Tooltip component', () => {
    const indexPath = join(import.meta.dir, 'index.ts');
    const content = readFileSync(indexPath, 'utf-8');
    expect(content).toContain('Tooltip');
    expect(content).toContain('TooltipProps');
  });
});

describe('Tooltip component structure', () => {
  test('Tooltip has ARIA attributes for accessibility', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('role="tooltip"');
    expect(content).toContain('aria-describedby');
  });

  test('Tooltip has keyboard accessibility (Escape key)', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('Escape');
    expect(content).toContain('handleEscape');
  });

  test('Tooltip supports hover trigger mode', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('handleMouseEnter');
    expect(content).toContain('handleMouseLeave');
    expect(content).toContain('handleFocus');
    expect(content).toContain('handleBlur');
  });

  test('Tooltip supports click trigger mode', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('handleClick');
    expect(content).toContain('triggerMode');
  });

  test('Tooltip has configurable delay', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('delay');
    expect(content).toContain('startTimer');
    expect(content).toContain('clearTimer');
  });

  test('Tooltip has positioning logic', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('updatePosition');
    expect(content).toContain('getBoundingClientRect');
    expect(content).toContain('setPosition');
  });

  test('Tooltip handles click outside', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('handleClickOutside');
    expect(content).toContain('closeOnClickOutside');
  });

  test('Tooltip manages focus return on close', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('triggerRef()?.focus()');
  });
});

describe('Tooltip component styles', () => {
  test('Tooltip has trigger wrapper styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.omm-tooltip-trigger');
  });

  test('Tooltip has content styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.omm-tooltip');
  });

  test('Tooltip has fixed positioning', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('position: fixed');
    expect(content).toContain('z-index: 1000');
  });

  test('Tooltip has animation', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('omm-tooltip-in');
    expect(content).toContain('@keyframes omm-tooltip-in');
  });

  test('Tooltip uses theme tokens (no raw colors)', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('rgb(var(--color-surface))');
    expect(content).toContain('rgb(var(--color-line))');
    expect(content).toContain('rgb(var(--color-fg))');
  });
});

describe('Tooltip component props interface', () => {
  test('TooltipProps includes content prop', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('content: string');
  });

  test('TooltipProps includes delay prop', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('delay?: number');
  });

  test('TooltipProps includes trigger prop', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain("trigger?: 'hover' | 'click'");
  });

  test('TooltipProps includes closeOnClickOutside prop', () => {
    const tooltipPath = join(import.meta.dir, 'Tooltip.tsx');
    const content = readFileSync(tooltipPath, 'utf-8');
    expect(content).toContain('closeOnClickOutside?: boolean');
  });
});
