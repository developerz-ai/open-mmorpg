import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

describe('Tabs component exports', () => {
  test('Tabs.tsx exists and contains Tabs exports', () => {
    const tabsPath = join(import.meta.dir, 'Tabs.tsx');
    const content = readFileSync(tabsPath, 'utf-8');
    expect(content).toContain('export function Tabs');
    expect(content).toContain('export function TabList');
    expect(content).toContain('export function Tab');
    expect(content).toContain('export function TabPanel');
  });

  test('index.ts exports Tabs components', () => {
    const indexPath = join(import.meta.dir, 'index.ts');
    const content = readFileSync(indexPath, 'utf-8');
    expect(content).toContain('Tabs');
    expect(content).toContain('TabList');
    expect(content).toContain('Tab');
    expect(content).toContain('TabPanel');
    expect(content).toContain('TabsProps');
    expect(content).toContain('TabListProps');
    expect(content).toContain('TabProps');
    expect(content).toContain('TabPanelProps');
  });
});

describe('Tabs component structure', () => {
  test('Tabs has ARIA attributes for accessibility', () => {
    const tabsPath = join(import.meta.dir, 'Tabs.tsx');
    const content = readFileSync(tabsPath, 'utf-8');
    expect(content).toContain('role="tablist"');
    expect(content).toContain('role="tab"');
    expect(content).toContain('role="tabpanel"');
    expect(content).toContain('aria-selected');
    expect(content).toContain('aria-controls');
    expect(content).toContain('aria-labelledby');
  });

  test('Tabs has keyboard navigation', () => {
    const tabsPath = join(import.meta.dir, 'Tabs.tsx');
    const content = readFileSync(tabsPath, 'utf-8');
    expect(content).toContain('ArrowRight');
    expect(content).toContain('ArrowLeft');
    expect(content).toContain('Home');
    expect(content).toContain('End');
    expect(content).toContain('Enter');
    expect(content).toContain('Space');
    expect(content).toContain('focusNext');
    expect(content).toContain('focusFirst');
    expect(content).toContain('focusLast');
  });

  test('Tabs uses context for compound components', () => {
    const tabsPath = join(import.meta.dir, 'Tabs.tsx');
    const content = readFileSync(tabsPath, 'utf-8');
    expect(content).toContain('TabsContext');
    expect(content).toContain('useTabsContext');
    expect(content).toContain('registerTab');
  });

  test('Tabs registers tabs for keyboard navigation', () => {
    const tabsPath = join(import.meta.dir, 'Tabs.tsx');
    const content = readFileSync(tabsPath, 'utf-8');
    expect(content).toContain('tabs:');
    expect(content).toContain('registerTab');
    expect(content).toContain('unregister');
  });
});

describe('Tabs component styles', () => {
  test('Tabs has tab-list styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.tab-list');
  });

  test('Tabs has tab button styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.tab');
    expect(content).toContain('.tab--selected');
  });

  test('Tabs has tab-panel styles', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('.tab-panel');
  });

  test('Tabs has bottom border for tab list', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('border-bottom');
  });

  test('Tabs has selected tab indicator', () => {
    const cssPath = join(import.meta.dir, 'components.css');
    const content = readFileSync(cssPath, 'utf-8');
    expect(content).toContain('border-bottom-color');
  });
});
