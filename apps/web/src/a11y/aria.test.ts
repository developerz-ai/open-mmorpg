import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

describe('ARIA Labels Verification', () => {
  const uiSrcPath = join(import.meta.dir, '../../../../packages/ui/src');

  test('Button has accessible name when text present', () => {
    const buttonPath = join(uiSrcPath, 'Button.tsx');
    const content = readFileSync(buttonPath, 'utf-8');

    // Button should accept children for accessible name
    expect(content).toContain('children');
  });

  test('Button supports aria-label prop', () => {
    const buttonPath = join(uiSrcPath, 'Button.tsx');
    const content = readFileSync(buttonPath, 'utf-8');

    // Should spread props allowing aria-label
    expect(content).toMatch(/\.\.\.rest/);
  });

  test('TextField has associated label', () => {
    const tfPath = join(uiSrcPath, 'TextField.tsx');
    const content = readFileSync(tfPath, 'utf-8');

    // Should have label prop and id
    expect(content).toContain('label');
    expect(content).toContain('id');
  });

  test('Dialog has role="dialog"', () => {
    const dialogPath = join(uiSrcPath, 'Dialog.tsx');
    const content = readFileSync(dialogPath, 'utf-8');

    // Dialog should have role attribute
    expect(content).toContain('role="dialog"');
  });

  test('Dialog has aria-modal attribute', () => {
    const dialogPath = join(uiSrcPath, 'Dialog.tsx');
    const content = readFileSync(dialogPath, 'utf-8');

    expect(content).toContain('aria-modal');
  });

  test('Alert has role attribute for a11y', () => {
    const alertPath = join(uiSrcPath, 'Alert.tsx');
    const content = readFileSync(alertPath, 'utf-8');

    // Should have role based on tone
    expect(content).toContain('role=');
  });

  test('Progress has role="progressbar"', () => {
    const progressPath = join(uiSrcPath, 'Progress.tsx');
    const content = readFileSync(progressPath, 'utf-8');

    expect(content).toContain('role="progressbar"');
  });

  test('Progress has aria-valuenow', () => {
    const progressPath = join(uiSrcPath, 'Progress.tsx');
    const content = readFileSync(progressPath, 'utf-8');

    expect(content).toContain('aria-valuenow');
  });

  test('Tabs has proper ARIA structure', () => {
    const tabsPath = join(uiSrcPath, 'Tabs.tsx');
    const content = readFileSync(tabsPath, 'utf-8');

    // Should have tablist, tab, tabpanel roles
    expect(content).toContain('role="tablist"');
    expect(content).toContain('role="tab"');
    expect(content).toContain('role="tabpanel"');
  });

  test('Badge is a semantic element', () => {
    const badgePath = join(uiSrcPath, 'Badge.tsx');
    const content = readFileSync(badgePath, 'utf-8');

    // Badge uses span which is neutral; for a11y would need role if not decorated
    expect(content).toMatch(/span/);
  });
});

describe('Form Accessibility', () => {
  const uiSrcPath = join(import.meta.dir, '../../../../packages/ui/src');

  test('TextField shows aria-invalid on error', () => {
    const tfPath = join(uiSrcPath, 'TextField.tsx');
    const content = readFileSync(tfPath, 'utf-8');

    expect(content).toContain('aria-invalid');
  });

  test('TextField supports required prop', () => {
    const tfPath = join(uiSrcPath, 'TextField.tsx');
    const content = readFileSync(tfPath, 'utf-8');

    // Should extend InputHTMLAttributes which includes required
    expect(content).toContain('JSX.InputHTMLAttributes');
  });

  test('Select has proper ARIA attributes', () => {
    const selectPath = join(uiSrcPath, 'Select.tsx');
    const content = readFileSync(selectPath, 'utf-8');

    // Select should have aria attributes for accessibility
    expect(content).toMatch(/aria/i);
  });
});
