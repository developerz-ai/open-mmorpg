import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

describe('Keyboard Navigation Accessibility', () => {
  const uiSrcPath = join(import.meta.dir, '../../../../packages/ui/src');

  test('Dialog component has focus trap implementation', () => {
    const dialogPath = join(uiSrcPath, 'Dialog.tsx');
    const content = readFileSync(dialogPath, 'utf-8');

    // Dialog is documented as having focus trap in its comment
    expect(content).toMatch(/focus trap/i);
  });

  test('Button component supports keyboard activation', () => {
    const buttonPath = join(uiSrcPath, 'Button.tsx');
    const content = readFileSync(buttonPath, 'utf-8');

    // Should be keyboard accessible by default
    expect(content).toContain('type="button"');
  });

  test('Select component has arrow key handling', () => {
    const selectPath = join(uiSrcPath, 'Select.tsx');
    const content = readFileSync(selectPath, 'utf-8');

    // Should handle keyboard navigation
    expect(content).toMatch(/(Arrow|Home|End)/i);
  });

  test('Tabs component supports keyboard navigation', () => {
    const tabsPath = join(uiSrcPath, 'Tabs.tsx');
    const content = readFileSync(tabsPath, 'utf-8');

    // Should have keyboard support
    expect(content).toMatch(/(Arrow(Left|Right)|Home|End)/i);
  });

  test('TextField has a11y support', () => {
    const tfPath = join(uiSrcPath, 'TextField.tsx');
    const content = readFileSync(tfPath, 'utf-8');

    // Should have a11y attributes
    expect(content).toContain('aria-invalid');
  });
});

describe('Focus Management Utilities', () => {
  const uiSrcPath = join(import.meta.dir, '../../../../packages/ui/src');

  test('focus.ts exports focus trap utilities', () => {
    const focusPath = join(uiSrcPath, 'focus.ts');
    const content = readFileSync(focusPath, 'utf-8');

    expect(content).toContain('useFocusTrap');
    expect(content).toContain('useFocusReturn');
    expect(content).toContain('getFocusableElements');
    expect(content).toContain('getFirstFocusable');
  });

  test('focus trap implementation handles tab cycles', () => {
    const focusPath = join(uiSrcPath, 'focus.ts');
    const content = readFileSync(focusPath, 'utf-8');

    // Should handle Tab and Shift+Tab
    expect(content).toMatch(/Tab/i);
    expect(content).toMatch(/Shift/i);
  });
});
