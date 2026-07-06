import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

describe('Dialog component exports', () => {
  test('Dialog.tsx exists and contains Dialog exports', () => {
    const dialogPath = join(import.meta.dir, 'Dialog.tsx');
    const content = readFileSync(dialogPath, 'utf-8');
    expect(content).toContain('export function Dialog');
    expect(content).toContain('export function DialogTitle');
    expect(content).toContain('export function DialogContent');
    expect(content).toContain('export function DialogActions');
  });

  test('index.ts exports Dialog components', () => {
    const indexPath = join(import.meta.dir, 'index.ts');
    const content = readFileSync(indexPath, 'utf-8');
    expect(content).toContain('Dialog');
    expect(content).toContain('DialogTitle');
    expect(content).toContain('DialogContent');
    expect(content).toContain('DialogActions');
    expect(content).toContain('DialogProps');
    expect(content).toContain('DialogTitleProps');
    expect(content).toContain('DialogContentProps');
    expect(content).toContain('DialogActionsProps');
  });
});

describe('Dialog component structure', () => {
  test('Dialog has ARIA attributes for accessibility', () => {
    const dialogPath = join(import.meta.dir, 'Dialog.tsx');
    const content = readFileSync(dialogPath, 'utf-8');
    expect(content).toContain('role="dialog"');
    expect(content).toContain('aria-modal="true"');
    expect(content).toContain('aria-labelledby');
    expect(content).toContain('aria-describedby');
  });

  test('Dialog has focus trap', () => {
    const dialogPath = join(import.meta.dir, 'Dialog.tsx');
    const content = readFileSync(dialogPath, 'utf-8');
    expect(content).toContain('handleTab');
    expect(content).toContain('focusable');
    expect(content).toContain('Tab');
  });

  test('Dialog has escape key handling', () => {
    const dialogPath = join(import.meta.dir, 'Dialog.tsx');
    const content = readFileSync(dialogPath, 'utf-8');
    expect(content).toContain('Escape');
    expect(content).toContain('handleEscape');
  });

  test('Dialog has overlay click handling', () => {
    const dialogPath = join(import.meta.dir, 'Dialog.tsx');
    const content = readFileSync(dialogPath, 'utf-8');
    expect(content).toContain('handleOverlayClick');
    expect(content).toContain('overlayRef');
  });

  test('Dialog locks body scroll when open', () => {
    const dialogPath = join(import.meta.dir, 'Dialog.tsx');
    const content = readFileSync(dialogPath, 'utf-8');
    expect(content).toContain('document.body.style.overflow');
  });

  test('Dialog restores focus on close', () => {
    const dialogPath = join(import.meta.dir, 'Dialog.tsx');
    const content = readFileSync(dialogPath, 'utf-8');
    expect(content).toContain('previouslyFocused');
  });
});

describe('Dialog component styles', () => {
  test('Dialog has overlay styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.dialog-overlay');
  });

  test('Dialog has content styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.dialog-content');
  });

  test('Dialog has title styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.dialog-title');
  });

  test('Dialog has body styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.dialog-body');
  });

  test('Dialog has actions styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.dialog-actions');
  });
});
