import type { ToastTone } from '@omm/ui';
import { createSignal, type JSX } from 'solid-js';

export interface ToastMessage {
  id: string;
  tone?: ToastTone;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
  children: JSX.Element;
}

/**
 * Toast notification hook.
 * Manages toast state for the app-wide ToastContainer.
 *
 * @example
 * ```tsx
 * const toasts = createSignal<ToastItem[]>([]);
 *
 * function MyApp() {
 *   const toast = useToast(toasts);
 *   return (
 *     <>
 *       <button onClick={() => toast.show('Success!', 'success')}>
 *         Show Toast
 *       </button>
 *       <ToastContainer toasts={toasts()} onDismiss={(id) => ...} />
 *     </>
 *   );
 * }
 * ```
 */
export function createToastManager() {
  const [toasts, setToasts] = createSignal<ToastMessage[]>([]);

  const show = (
    message: JSX.Element,
    tone: ToastTone = 'info',
    duration = 5000,
    action?: ToastMessage['action'],
  ): void => {
    const id = crypto.randomUUID();
    const toast: ToastMessage = {
      id,
      tone,
      duration,
      action,
      children: message,
    };

    setToasts((prev) => [...prev, toast]);
  };

  const dismiss = (id: string): void => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  return {
    toasts,
    show,
    dismiss,
  };
}

/**
 * Hook that returns toast control functions.
 * Use this in components to show toasts via the app-wide toast manager.
 *
 * @param manager - The toast manager from the app root
 */
export function useToast(manager: ReturnType<typeof createToastManager>) {
  return {
    show: manager.show,
    dismiss: manager.dismiss,
    success: (message: JSX.Element, duration?: number) =>
      manager.show(message, 'success', duration),
    error: (message: JSX.Element, duration?: number) => manager.show(message, 'error', duration),
    warning: (message: JSX.Element, duration?: number) =>
      manager.show(message, 'warning', duration),
    info: (message: JSX.Element, duration?: number) => manager.show(message, 'info', duration),
  };
}
