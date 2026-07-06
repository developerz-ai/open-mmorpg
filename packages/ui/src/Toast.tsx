import type { JSX } from 'solid-js';
import { createEffect, createSignal, Index, Show, splitProps } from 'solid-js';
import { cx } from './cx.ts';

export type ToastTone = 'success' | 'error' | 'warning' | 'info';

export interface ToastProps {
  /** Tone/severity for visual styling. */
  tone?: ToastTone;
  /** Auto-dismiss after ms (0 = no auto-dismiss). Default: 5000. */
  duration?: number;
  /** Optional primary action button. */
  action?: {
    label: string;
    onClick: () => void;
  };
  /** Called when toast is dismissed (auto-dismiss or close button). */
  onDismiss?: () => void;
  class?: string;
  children: JSX.Element;
}

/**
 * An individual toast notification. Auto-dismisses after duration (default 5s),
 * with optional action button. Managed by ToastContainer.
 */
export function Toast(props: ToastProps): JSX.Element {
  const [local, rest] = splitProps(props, [
    'tone',
    'duration',
    'action',
    'onDismiss',
    'class',
    'children',
  ]);

  const tone = () => local.tone ?? 'info';
  const duration = () => local.duration ?? 5000;
  const [isVisible, setIsVisible] = createSignal(true);

  // Auto-dismiss timer
  createEffect(() => {
    if (duration() === 0) return;
    const timer = setTimeout(() => {
      handleDismiss();
    }, duration());
    return () => clearTimeout(timer);
  });

  const handleDismiss = () => {
    setIsVisible(false);
    // Wait for exit animation before calling onDismiss
    setTimeout(() => {
      local.onDismiss?.();
    }, 150);
  };

  const handleAction = () => {
    local.action?.onClick();
    handleDismiss();
  };

  return (
    <div
      class={cx('omm-toast', `omm-toast--${tone()}`, local.class)}
      classList={{ 'omm-toast--exiting': !isVisible() }}
      role="status"
      aria-live="polite"
      {...rest}
    >
      <div class="omm-toast__content">{local.children}</div>
      <div class="omm-toast__actions">
        <Show when={local.action}>
          <button type="button" class="omm-toast__action" onClick={handleAction}>
            {local.action?.label}
          </button>
        </Show>
        <button
          type="button"
          class="omm-toast__close"
          onClick={handleDismiss}
          aria-label="Dismiss notification"
        >
          ×
        </button>
      </div>
    </div>
  );
}

export interface ToastItem {
  id: string;
  tone?: ToastTone;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
  children: JSX.Element;
}

export interface ToastContainerProps {
  /** Toasts to display. */
  toasts: ToastItem[];
  /** Called when a toast is dismissed. */
  onDismiss: (id: string) => void;
  /** Position in viewport. Default: 'bottom-right'. */
  position?: 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left';
  class?: string;
}

/**
 * Container for toast notifications. Renders toasts in a corner stack with
 * enter/exit animations. Manage toasts array in parent state.
 */
export function ToastContainer(props: ToastContainerProps): JSX.Element {
  const [local, rest] = splitProps(props, ['toasts', 'onDismiss', 'position', 'class']);

  const position = () => local.position ?? 'bottom-right';

  return (
    <div
      class={cx('omm-toast-container', `omm-toast-container--${position()}`, local.class)}
      {...rest}
    >
      <Index each={local.toasts}>
        {(toast) => (
          <Toast
            tone={toast().tone}
            duration={toast().duration}
            action={toast().action}
            onDismiss={() => local.onDismiss(toast().id)}
          >
            {toast().children}
          </Toast>
        )}
      </Index>
    </div>
  );
}
