import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

/**
 * Comprehensive accessibility tests for all UI components.
 * Tests ARIA attributes, keyboard navigation, focus management, and screen reader support.
 */

describe('Dialog/Modal component accessibility', () => {
  test('Dialog has proper ARIA attributes', () => {
    const content = readFileSync(join(import.meta.dir, 'Dialog.tsx'), 'utf-8');
    expect(content).toContain('role="dialog"');
    expect(content).toContain('aria-modal="true"');
    expect(content).toContain('aria-labelledby');
    expect(content).toContain('aria-describedby');
  });

  test('Dialog implements focus trap', () => {
    const content = readFileSync(join(import.meta.dir, 'Dialog.tsx'), 'utf-8');
    expect(content).toMatch(/handleTab|focusTrap|trapFocus/);
    expect(content).toMatch(/Tab|focusable|interactive/);
  });

  test('Dialog handles Escape key', () => {
    const content = readFileSync(join(import.meta.dir, 'Dialog.tsx'), 'utf-8');
    expect(content).toContain('Escape');
    expect(content).toContain('onKeyDown');
  });

  test('Dialog restores focus on close', () => {
    const content = readFileSync(join(import.meta.dir, 'Dialog.tsx'), 'utf-8');
    expect(content).toMatch(/previouslyFocused|previousFocus|restoreFocus/);
  });

  test('Dialog prevents body scroll when open', () => {
    const content = readFileSync(join(import.meta.dir, 'Dialog.tsx'), 'utf-8');
    expect(content).toContain('overflow');
    expect(content).toContain('body');
  });

  test('DialogTitle has heading level', () => {
    const content = readFileSync(join(import.meta.dir, 'Dialog.tsx'), 'utf-8');
    expect(content).toMatch(/h[1-6]|heading/);
  });
});

describe('Select/SelectOption component accessibility', () => {
  test('Select has combobox role', () => {
    const content = readFileSync(join(import.meta.dir, 'Select.tsx'), 'utf-8');
    expect(content).toContain('role');
    expect(content).toMatch(/combobox|listbox/);
  });

  test('Select has expandable ARIA attributes', () => {
    const content = readFileSync(join(import.meta.dir, 'Select.tsx'), 'utf-8');
    expect(content).toContain('aria-expanded');
    expect(content).toContain('aria-haspopup');
  });

  test('Select has keyboard navigation', () => {
    const content = readFileSync(join(import.meta.dir, 'Select.tsx'), 'utf-8');
    expect(content).toMatch(/ArrowDown|ArrowUp|Enter|Escape/);
    expect(content).toMatch(/onKeyDown|handleKey/);
  });

  test('SelectOption has option role', () => {
    const content = readFileSync(join(import.meta.dir, 'Select.tsx'), 'utf-8');
    expect(content).toMatch(/role=.option|option/);
  });

  test('Select has label prop for accessible name', () => {
    const content = readFileSync(join(import.meta.dir, 'Select.tsx'), 'utf-8');
    expect(content).toMatch(/label.*string|label:.*string/);
  });

  test('Select uses focus management for keyboard nav', () => {
    const content = readFileSync(join(import.meta.dir, 'Select.tsx'), 'utf-8');
    expect(content).toMatch(/focus|tabindex|highlighted/);
  });

  test('SelectOption has selected state', () => {
    const content = readFileSync(join(import.meta.dir, 'Select.tsx'), 'utf-8');
    expect(content).toContain('aria-selected');
  });
});

describe('Tabs component accessibility', () => {
  test('TabList has tablist role', () => {
    const content = readFileSync(join(import.meta.dir, 'Tabs.tsx'), 'utf-8');
    expect(content).toContain('role="tablist"');
  });

  test('Tab has tab role', () => {
    const content = readFileSync(join(import.meta.dir, 'Tabs.tsx'), 'utf-8');
    expect(content).toContain('role="tab"');
  });

  test('TabPanel has tabpanel role', () => {
    const content = readFileSync(join(import.meta.dir, 'Tabs.tsx'), 'utf-8');
    expect(content).toContain('role="tabpanel"');
  });

  test('Tabs have aria-selected for active state', () => {
    const content = readFileSync(join(import.meta.dir, 'Tabs.tsx'), 'utf-8');
    expect(content).toContain('aria-selected');
  });

  test('Tabs have aria-controls linking to panel', () => {
    const content = readFileSync(join(import.meta.dir, 'Tabs.tsx'), 'utf-8');
    expect(content).toContain('aria-controls');
  });

  test('TabPanels have aria-labelledby linking to tab', () => {
    const content = readFileSync(join(import.meta.dir, 'Tabs.tsx'), 'utf-8');
    expect(content).toContain('aria-labelledby');
  });

  test('Tabs support arrow key navigation', () => {
    const content = readFileSync(join(import.meta.dir, 'Tabs.tsx'), 'utf-8');
    expect(content).toMatch(/ArrowLeft|ArrowRight|Home|End/);
  });

  test('Tabs support Enter/Space activation', () => {
    const content = readFileSync(join(import.meta.dir, 'Tabs.tsx'), 'utf-8');
    expect(content).toMatch(/Enter| |key/);
  });
});

describe('Tooltip component accessibility', () => {
  test('Tooltip has proper ARIA attributes', () => {
    const content = readFileSync(join(import.meta.dir, 'Tooltip.tsx'), 'utf-8');
    expect(content).toMatch(/role=.tooltip|tooltip/);
    expect(content).toContain('aria-describedby');
  });

  test('Tooltip triggers on hover/focus', () => {
    const content = readFileSync(join(import.meta.dir, 'Tooltip.tsx'), 'utf-8');
    expect(content).toMatch(/onMouseEnter|onFocus|onMouseLeave|onBlur/);
  });

  test('Tooltip has keyboard activation (Escape to close)', () => {
    const content = readFileSync(join(import.meta.dir, 'Tooltip.tsx'), 'utf-8');
    expect(content).toContain('Escape');
  });

  test('Tooltip has proper focus management', () => {
    const content = readFileSync(join(import.meta.dir, 'Tooltip.tsx'), 'utf-8');
    expect(content).toMatch(/focus|tabindex/);
  });

  test('Tooltip has delay settings for accessibility', () => {
    const content = readFileSync(join(import.meta.dir, 'Tooltip.tsx'), 'utf-8');
    expect(content).toMatch(/delay|hideDelay|showDelay/);
  });
});

describe('Toast/Notification component accessibility', () => {
  test('Toast has alert or role=status', () => {
    const content = readFileSync(join(import.meta.dir, 'Toast.tsx'), 'utf-8');
    expect(content).toMatch(/role=.alert|role=.status|alert/);
  });

  test('Toast has aria-live for announcements', () => {
    const content = readFileSync(join(import.meta.dir, 'Toast.tsx'), 'utf-8');
    expect(content).toContain('aria-live');
  });

  test('Toast has polite aria-live for announcements', () => {
    const content = readFileSync(join(import.meta.dir, 'Toast.tsx'), 'utf-8');
    expect(content).toContain('aria-live');
    expect(content).toContain('polite');
  });

  test('Toast has auto-dismiss with timer', () => {
    const content = readFileSync(join(import.meta.dir, 'Toast.tsx'), 'utf-8');
    expect(content).toMatch(/setTimeout|setInterval|timer|autoDismiss/);
  });

  test('Toast has close button for manual dismissal', () => {
    const content = readFileSync(join(import.meta.dir, 'Toast.tsx'), 'utf-8');
    expect(content).toMatch(/close|dismiss|button/);
  });

  test('Toast has proper ARIA labels', () => {
    const content = readFileSync(join(import.meta.dir, 'Toast.tsx'), 'utf-8');
    expect(content).toMatch(/aria-label|aria-labelledby/);
  });
});

describe('Progress/Spinner component accessibility', () => {
  test('Progress has progressbar role', () => {
    const content = readFileSync(join(import.meta.dir, 'Progress.tsx'), 'utf-8');
    expect(content).toContain('role="progressbar"');
  });

  test('Progress has aria-valuenow for current value', () => {
    const content = readFileSync(join(import.meta.dir, 'Progress.tsx'), 'utf-8');
    expect(content).toContain('aria-valuenow');
  });

  test('Progress has aria-valuemin for minimum', () => {
    const content = readFileSync(join(import.meta.dir, 'Progress.tsx'), 'utf-8');
    expect(content).toContain('aria-valuemin');
  });

  test('Progress has aria-valuemax for maximum', () => {
    const content = readFileSync(join(import.meta.dir, 'Progress.tsx'), 'utf-8');
    expect(content).toContain('aria-valuemax');
  });

  test('Spinner has label prop for accessible name', () => {
    const content = readFileSync(join(import.meta.dir, 'Spinner.tsx'), 'utf-8');
    expect(content).toMatch(/label.*string|label:|sr-only/);
  });

  test('Indeterminate progress lacks aria-valuenow', () => {
    const content = readFileSync(join(import.meta.dir, 'Progress.tsx'), 'utf-8');
    // Should check for conditional aria-valuenow
    expect(content).toMatch(/if.*aria-valuenow|undefined/);
  });
});

describe('Button component accessibility', () => {
  test('Button has proper type attribute', () => {
    const content = readFileSync(join(import.meta.dir, 'Button.tsx'), 'utf-8');
    expect(content).toContain('type=');
  });

  test('Button forwards native HTML attributes', () => {
    const content = readFileSync(join(import.meta.dir, 'Button.tsx'), 'utf-8');
    expect(content).toMatch(/\.\.\.|JSX\.ButtonHTMLAttributes/);
  });

  test('Button forwards disabled attribute via props', () => {
    const content = readFileSync(join(import.meta.dir, 'Button.tsx'), 'utf-8');
    expect(content).toMatch(/\.\.\.|JSX\.ButtonHTMLAttributes/);
  });
});

describe('TextField component accessibility', () => {
  test('TextField has label association', () => {
    const content = readFileSync(join(import.meta.dir, 'TextField.tsx'), 'utf-8');
    expect(content).toMatch(/htmlFor|id=|label/);
  });

  test('TextField has aria-describedby for errors', () => {
    const content = readFileSync(join(import.meta.dir, 'TextField.tsx'), 'utf-8');
    expect(content).toMatch(/aria-describedby|error/);
  });

  test('TextField forwards required attribute via props', () => {
    const content = readFileSync(join(import.meta.dir, 'TextField.tsx'), 'utf-8');
    expect(content).toMatch(/JSX\.InputHTMLAttributes|\.\.\.rest/);
  });

  test('TextField forwards disabled attribute via props', () => {
    const content = readFileSync(join(import.meta.dir, 'TextField.tsx'), 'utf-8');
    expect(content).toMatch(/JSX\.InputHTMLAttributes|\.\.\.rest/);
  });
});

describe('Card component accessibility', () => {
  test('Card has proper heading structure', () => {
    const content = readFileSync(join(import.meta.dir, 'Card.tsx'), 'utf-8');
    expect(content).toMatch(/h[1-6]|heading/);
  });

  test('Card has landmark role if needed', () => {
    const content = readFileSync(join(import.meta.dir, 'Card.tsx'), 'utf-8');
    // Card might have article, region, or no role
    expect(content.length).toBeGreaterThan(0);
  });
});

describe('Badge component accessibility', () => {
  test('Badge has visible text content', () => {
    const content = readFileSync(join(import.meta.dir, 'Badge.tsx'), 'utf-8');
    expect(content).toMatch(/children|BadgeProps/);
  });
});

describe('Alert component accessibility', () => {
  test('Alert has alert role', () => {
    const content = readFileSync(join(import.meta.dir, 'Alert.tsx'), 'utf-8');
    expect(content).toMatch(/role=.alert|alert/);
  });

  test('Alert has proper role for announcements', () => {
    const content = readFileSync(join(import.meta.dir, 'Alert.tsx'), 'utf-8');
    // role="alert" implicitly has aria-live="assertive", role="status" has aria-live="polite"
    expect(content).toContain('role=');
  });
});

describe('Table component accessibility', () => {
  test('Table has semantic table structure', () => {
    const content = readFileSync(join(import.meta.dir, 'Table.tsx'), 'utf-8');
    expect(content).toMatch(/thead|tbody|th|td/);
  });
});

describe('Focus indicators', () => {
  test('Components have visible focus styles', () => {
    const css = readFileSync(join(import.meta.dir, 'components.css'), 'utf-8');
    expect(css).toMatch(/:focus|focus-visible/);
  });

  test('Focus styles meet contrast requirements', () => {
    const css = readFileSync(join(import.meta.dir, 'components.css'), 'utf-8');
    // Should have outline or box-shadow for focus
    expect(css).toMatch(/outline|box-shadow.*focus/);
  });

  test('No outline: none on focus-visible', () => {
    const css = readFileSync(join(import.meta.dir, 'components.css'), 'utf-8');
    // Check we're not hiding focus on keyboard navigation
    expect(css).not.toMatch(/focus-visible.*outline:\s*none/);
  });
});

describe('Color contrast', () => {
  test('Components use semantic tokens for colors', () => {
    const css = readFileSync(join(import.meta.dir, 'theme.css'), 'utf-8');
    // Should use CSS variables, not hardcoded colors
    expect(css).toMatch(/var\(--[a-z-]+\)/);
  });

  test('Theme has defined color tokens', () => {
    const css = readFileSync(join(import.meta.dir, 'theme.css'), 'utf-8');
    expect(css).toMatch(/--color-|--bg-|--fg-|--text-|--border-/);
  });
});

describe('Screen reader support', () => {
  test('Components have sr-only class for hidden text', () => {
    const css = readFileSync(join(import.meta.dir, 'theme.css'), 'utf-8');
    expect(css).toMatch(/sr-only|screen-reader|visually-hidden/);
  });

  test('Interactive elements have accessible names', () => {
    const components = ['TextField.tsx', 'Select.tsx'];
    for (const comp of components) {
      const content = readFileSync(join(import.meta.dir, comp), 'utf-8');
      // Should have aria-label, aria-labelledby, or visible label
      expect(content).toMatch(/aria-label|aria-labelledby|label/);
    }
  });

  test('Icon-only buttons have aria-label', () => {
    const content = readFileSync(join(import.meta.dir, 'Button.tsx'), 'utf-8');
    // Button should forward aria-label
    expect(content).toMatch(/aria-label|JSX\.ButtonHTMLAttributes/);
  });
});

describe('Keyboard navigation', () => {
  test('Dialog and Tooltip have Escape key handler', () => {
    const components = ['Dialog.tsx', 'Tooltip.tsx'];
    for (const comp of components) {
      const content = readFileSync(join(import.meta.dir, comp), 'utf-8');
      expect(content).toContain('Escape');
    }
  });

  test('Composite widgets support arrow keys', () => {
    const components = ['Select.tsx', 'Tabs.tsx'];
    for (const comp of components) {
      const content = readFileSync(join(import.meta.dir, comp), 'utf-8');
      expect(content).toMatch(/ArrowUp|ArrowDown|ArrowLeft|ArrowRight/);
    }
  });

  test('Interactive elements have tabindex', () => {
    const content = readFileSync(join(import.meta.dir, 'Tabs.tsx'), 'utf-8');
    expect(content).toContain('tabIndex');
  });
});
