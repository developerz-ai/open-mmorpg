import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

describe('Toast component exports', () => {
  test('Toast.tsx exists and contains Toast exports', () => {
    const toastPath = join(import.meta.dir, 'Toast.tsx');
    const content = readFileSync(toastPath, 'utf-8');
    expect(content).toContain('export function Toast');
    expect(content).toContain('export function ToastContainer');
    expect(content).toContain('export interface ToastProps');
    expect(content).toContain('export interface ToastItem');
    expect(content).toContain('export interface ToastContainerProps');
    expect(content).toContain('export type ToastTone');
  });

  test('index.ts exports Toast components', () => {
    const indexPath = join(import.meta.dir, 'index.ts');
    const content = readFileSync(indexPath, 'utf-8');
    expect(content).toContain('Toast');
    expect(content).toContain('ToastContainer');
    expect(content).toContain('ToastProps');
    expect(content).toContain('ToastItem');
    expect(content).toContain('ToastContainerProps');
    expect(content).toContain('ToastTone');
  });
});

describe('Toast component structure', () => {
  test('Toast has auto-dismiss timer', () => {
    const toastPath = join(import.meta.dir, 'Toast.tsx');
    const content = readFileSync(toastPath, 'utf-8');
    expect(content).toContain('duration');
    expect(content).toContain('setTimeout');
    expect(content).toContain('auto-dismiss');
  });

  test('Toast has onDismiss callback', () => {
    const toastPath = join(import.meta.dir, 'Toast.tsx');
    const content = readFileSync(toastPath, 'utf-8');
    expect(content).toContain('onDismiss');
    expect(content).toContain('handleDismiss');
  });

  test('Toast has action button support', () => {
    const toastPath = join(import.meta.dir, 'Toast.tsx');
    const content = readFileSync(toastPath, 'utf-8');
    expect(content).toContain('action');
    expect(content).toContain('handleAction');
    expect(content).toContain('onClick');
  });

  test('Toast has tone variants', () => {
    const toastPath = join(import.meta.dir, 'Toast.tsx');
    const content = readFileSync(toastPath, 'utf-8');
    expect(content).toContain('success');
    expect(content).toContain('error');
    expect(content).toContain('warning');
    expect(content).toContain('info');
  });

  test('Toast has exit animation', () => {
    const toastPath = join(import.meta.dir, 'Toast.tsx');
    const content = readFileSync(toastPath, 'utf-8');
    expect(content).toContain('isVisible');
    expect(content).toContain('exiting');
    expect(content).toContain('omm-toast--exiting');
  });

  test('Toast has accessibility attributes', () => {
    const toastPath = join(import.meta.dir, 'Toast.tsx');
    const content = readFileSync(toastPath, 'utf-8');
    expect(content).toContain('role="status"');
    expect(content).toContain('aria-live="polite"');
  });

  test('ToastContainer manages toasts array', () => {
    const toastPath = join(import.meta.dir, 'Toast.tsx');
    const content = readFileSync(toastPath, 'utf-8');
    expect(content).toContain('toasts');
    expect(content).toContain('onDismiss');
    expect(content).toContain('toast.id');
  });

  test('ToastContainer has position variants', () => {
    const toastPath = join(import.meta.dir, 'Toast.tsx');
    const content = readFileSync(toastPath, 'utf-8');
    expect(content).toContain('bottom-right');
    expect(content).toContain('bottom-left');
    expect(content).toContain('top-right');
    expect(content).toContain('top-left');
  });
});

describe('Toast component styles', () => {
  test('Toast has base styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.omm-toast');
  });

  test('Toast has tone variants', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.omm-toast--success');
    expect(content).toContain('.omm-toast--error');
    expect(content).toContain('.omm-toast--warning');
    expect(content).toContain('.omm-toast--info');
  });

  test('Toast has content and actions styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.omm-toast__content');
    expect(content).toContain('.omm-toast__actions');
    expect(content).toContain('.omm-toast__action');
    expect(content).toContain('.omm-toast__close');
  });

  test('Toast has exit animation style', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.omm-toast--exiting');
    expect(content).toContain('opacity: 0');
  });

  test('Toast has enter animation', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('@keyframes omm-toast-in');
    expect(content).toContain('translateY');
  });

  test('ToastContainer has base styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.omm-toast-container');
    expect(content).toContain('position: fixed');
    expect(content).toContain('z-index: 1100');
  });

  test('ToastContainer has position variants', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.omm-toast-container--top-right');
    expect(content).toContain('.omm-toast-container--top-left');
    expect(content).toContain('.omm-toast-container--bottom-right');
    expect(content).toContain('.omm-toast-container--bottom-left');
  });
});
