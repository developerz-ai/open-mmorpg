import type { JSX } from 'solid-js';
import { createContext, createEffect, createSignal, Show, splitProps, useContext } from 'solid-js';
import { cx } from './cx.ts';

interface DialogContextValue {
  close: () => void;
  dismissable: () => boolean;
}

const DialogContext = createContext<DialogContextValue>();

function useDialogContext(): DialogContextValue {
  const ctx = useContext(DialogContext);
  if (!ctx) throw new Error('Dialog compound components must be used inside Dialog');
  return ctx;
}

export interface DialogProps extends Omit<JSX.HTMLAttributes<HTMLDivElement>, 'isOpen'> {
  /** Whether the dialog is open. */
  isOpen: boolean;
  /** Called when dialog should close (button click, escape key, overlay click). */
  onClose: () => void;
  /** Optional title for accessibility (set via DialogTitle). */
  titleId?: string;
  /** Optional description ID for accessibility. */
  descriptionId?: string;
  /** Whether clicking the overlay should close the dialog. Default: true. */
  dismissable?: boolean;
}

/**
 * A modal dialog with overlay, focus trap, and escape key handling. Compound
 * component: use DialogTitle and DialogContent as children. Fully controlled:
 * parent owns open state via isOpen/onClose.
 */
export function Dialog(props: DialogProps) {
  const [local, rest] = splitProps(props, [
    'isOpen',
    'onClose',
    'titleId',
    'descriptionId',
    'dismissable',
    'class',
    'children',
  ]);

  const [overlayRef, setOverlayRef] = createSignal<HTMLDivElement>();
  const [dialogRef, setDialogContentRef] = createSignal<HTMLDivElement>();

  const dismissable = () => local.dismissable ?? true;
  const close = () => {
    if (dismissable()) {
      local.onClose();
    }
  };

  const contextValue: DialogContextValue = { close, dismissable };

  // Handle escape key
  createEffect(() => {
    if (!local.isOpen) return;
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        close();
      }
    };
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  });

  // Focus trap and body scroll lock
  createEffect((prevOnOpen) => {
    const isOpen = local.isOpen;
    const wasOpen = prevOnOpen ?? false;

    if (isOpen && !wasOpen) {
      // Dialog opened: save previously focused element, lock body scroll, trap focus
      const previouslyFocused = document.activeElement as HTMLElement | null;
      const dialog = dialogRef();

      if (dialog) {
        // Find first focusable element
        const focusable = dialog.querySelector<
          HTMLButtonElement | HTMLAnchorElement | HTMLInputElement
        >(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
        ) as HTMLElement | null;
        focusable?.focus();
      }

      document.body.style.overflow = 'hidden';

      return () => previouslyFocused;
    } else if (!isOpen && wasOpen) {
      // Dialog closed: restore body scroll, return focus to trigger
      document.body.style.overflow = '';
      const previouslyFocused = prevOnOpen as HTMLElement | null;
      queueMicrotask(() => previouslyFocused?.focus());
      return undefined;
    }
    return prevOnOpen;
  });

  // Focus trap: keep Tab cycles within dialog
  createEffect(() => {
    if (!local.isOpen) return;
    const dialog = dialogRef();
    if (!dialog) return;

    const handleTab = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;
      const focusable = dialog.querySelectorAll<
        HTMLButtonElement | HTMLAnchorElement | HTMLInputElement
      >('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])');
      const first = focusable[0] as HTMLElement;
      const last = focusable[focusable.length - 1] as HTMLElement;

      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    };

    dialog.addEventListener('keydown', handleTab);
    return () => dialog.removeEventListener('keydown', handleTab);
  });

  const handleOverlayClick = (e: MouseEvent) => {
    if (e.target === overlayRef()) {
      close();
    }
  };

  return (
    <Show when={local.isOpen}>
      <div
        ref={setOverlayRef}
        class={cx('dialog-overlay', local.class)}
        onClick={handleOverlayClick}
        {...rest}
      >
        <div
          ref={setDialogContentRef}
          role="dialog"
          aria-modal="true"
          aria-labelledby={local.titleId}
          aria-describedby={local.descriptionId}
          class="dialog-content"
        >
          <DialogContext.Provider value={contextValue}>{local.children}</DialogContext.Provider>
        </div>
      </div>
    </Show>
  );
}

export interface DialogTitleProps extends JSX.HTMLAttributes<HTMLHeadingElement> {
  /** Heading level (default: 2). */
  level?: 1 | 2 | 3 | 4 | 5 | 6;
}

/**
 * Dialog heading. Sets the accessible name; use exactly one per dialog.
 */
export function DialogTitle(props: DialogTitleProps) {
  const [local, rest] = splitProps(props, ['level', 'class', 'children', 'id']);
  const level = () => local.level ?? 2;
  const id = () => local.id ?? 'omm-dialog-title';

  const headingProps = () => ({
    id: id(),
    class: cx('dialog-title', local.class),
    ...rest,
  });

  // SolidJS requires explicit component mapping for dynamic tags
  if (level() === 1) return <h1 {...headingProps()}>{local.children}</h1>;
  if (level() === 2) return <h2 {...headingProps()}>{local.children}</h2>;
  if (level() === 3) return <h3 {...headingProps()}>{local.children}</h3>;
  if (level() === 4) return <h4 {...headingProps()}>{local.children}</h4>;
  if (level() === 5) return <h5 {...headingProps()}>{local.children}</h5>;
  return <h6 {...headingProps()}>{local.children}</h6>;
}

export interface DialogContentProps extends JSX.HTMLAttributes<HTMLDivElement> {}

/**
 * Dialog content area. Can contain any content; for scrollable content,
 * use native overflow:auto.
 */
export function DialogContent(props: DialogContentProps) {
  const [local, rest] = splitProps(props, ['class', 'children']);
  return (
    <div class={cx('dialog-body', local.class)} {...rest}>
      {local.children}
    </div>
  );
}

export interface DialogActionsProps extends JSX.HTMLAttributes<HTMLDivElement> {}

/**
 * Dialog footer for action buttons. Right-aligned by convention.
 */
export function DialogActions(props: DialogActionsProps) {
  const [local, rest] = splitProps(props, ['class', 'children']);
  return (
    <div class={cx('dialog-actions', local.class)} {...rest}>
      {local.children}
    </div>
  );
}
