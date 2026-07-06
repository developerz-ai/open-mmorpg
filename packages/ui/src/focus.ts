import { type Accessor, createEffect, onCleanup } from 'solid-js';
import { isServer } from 'solid-js/web';

/**
 * Get all focusable elements within a container.
 */
export function getFocusableElements(container: Element): HTMLElement[] {
  const focusableSelectors = [
    'a[href]',
    'button:not([disabled])',
    'textarea:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ].join(', ');

  return Array.from(container.querySelectorAll<HTMLElement>(focusableSelectors)).filter(
    (el) => getComputedStyle(el).display !== 'none' && getComputedStyle(el).visibility !== 'hidden',
  );
}

/**
 * Get the first focusable element in a container.
 */
export function getFirstFocusable(container: Element): HTMLElement | null {
  const elements = getFocusableElements(container);
  return elements[0] || null;
}

/**
 * Get the last focusable element in a container.
 */
export function getLastFocusable(container: Element): HTMLElement | null {
  const elements = getFocusableElements(container);
  return elements[elements.length - 1] || null;
}

/**
 * Focus trap hook - keeps focus within a container (for dialogs, modals, etc).
 * @param containerRef - Accessor to the container element
 * @param isEnabled - Whether the trap is active
 */
export function useFocusTrap(
  containerRef: Accessor<Element | undefined>,
  isEnabled: () => boolean,
) {
  if (isServer) return;

  createEffect(() => {
    const container = containerRef();
    const enabled = isEnabled();

    if (!container || !enabled) return;

    // Focus first element on mount
    const firstFocusable = getFirstFocusable(container);
    firstFocusable?.focus();

    const handleTabKey = (e: KeyboardEvent): void => {
      if (e.key !== 'Tab') return;

      const focusable = getFocusableElements(container);
      if (focusable.length === 0) return;

      const first = focusable[0];
      const last = focusable[focusable.length - 1];

      if (e.shiftKey) {
        // Shift+Tab: if focus is on first, move to last
        if (first && document.activeElement === first && last) {
          e.preventDefault();
          last.focus();
        }
      } else {
        // Tab: if focus is on last, move to first
        if (last && first && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };

    document.addEventListener('keydown', handleTabKey);
    onCleanup(() => document.removeEventListener('keydown', handleTabKey));
  });
}

/**
 * Focus return hook - restores focus to a previously focused element on cleanup.
 * Useful for dialogs, dropdowns, etc.
 * @param shouldCapture - When true, captures the current focus
 */
export function useFocusReturn(shouldCapture: () => boolean) {
  if (isServer) return () => {};

  let previousFocus: HTMLElement | null = null;

  createEffect(() => {
    if (shouldCapture()) {
      previousFocus = document.activeElement as HTMLElement;
    } else if (previousFocus) {
      // Restore focus when capture ends
      previousFocus.focus();
      previousFocus = null;
    }
  });

  // Return function to manually restore focus
  const restore = (): void => {
    if (previousFocus && typeof previousFocus.focus === 'function') {
      previousFocus.focus();
    }
  };

  return restore;
}

/**
 * Focus restore hook - captures focus on mount and restores on unmount.
 * Simpler variant of useFocusReturn for cleanup-only scenarios.
 * @param isEnabled - Whether to capture and restore focus
 */
export function useRestoreFocus(isEnabled: () => boolean) {
  if (isServer) return;

  let previousFocus: HTMLElement | null = null;

  createEffect(() => {
    if (!isEnabled()) return;

    // Capture current focus
    previousFocus = document.activeElement as HTMLElement;

    onCleanup(() => {
      // Restore focus on cleanup
      if (previousFocus && typeof previousFocus.focus === 'function') {
        // Use setTimeout to ensure DOM has settled
        setTimeout(() => {
          previousFocus?.focus();
        }, 0);
      }
    });
  });
}

/**
 * Auto-focus hook - focuses an element when a condition becomes true.
 * @param ref - Accessor to the element to focus
 * @param shouldFocus - When true, focuses the element
 */
export function useAutoFocus(ref: Accessor<HTMLElement | undefined>, shouldFocus: () => boolean) {
  if (isServer) return;

  createEffect(() => {
    const element = ref();
    const focus = shouldFocus();

    if (element && focus) {
      // Use setTimeout to ensure DOM is ready
      setTimeout(() => {
        element.focus();
      }, 0);
    }
  });
}
