import type { JSX } from 'solid-js';
import { createEffect, createMemo, createSignal, onCleanup, Show, splitProps } from 'solid-js';
import { cx } from './cx.ts';

export interface TooltipProps extends Omit<JSX.ButtonHTMLAttributes<HTMLButtonElement>, 'title'> {
  /** Tooltip content text. */
  content: string;
  /** Delay before showing on hover (ms). Default: 200. */
  delay?: number;
  /** Whether tooltip closes on click outside. Default: true. */
  closeOnClickOutside?: boolean;
  /**
   * Trigger behavior. 'hover' shows on mouse enter/focus, 'click' toggles on click/Enter.
   * Default: 'hover'.
   */
  trigger?: 'hover' | 'click';
}

/**
 * Hover or click tooltip with configurable delay and keyboard accessibility.
 * Supports 'hover' (mouse enter/focus) and 'click' (click/Enter) triggers.
 * Escape key closes, focus moves to trigger on close. Position-aware placement.
 */
export function Tooltip(props: TooltipProps) {
  const [local, rest] = splitProps(props, [
    'content',
    'delay',
    'closeOnClickOutside',
    'trigger',
    'class',
    'children',
  ]);

  const delay = () => local.delay ?? 200;
  const triggerMode = () => local.trigger ?? 'hover';
  const [isOpen, setIsOpen] = createSignal(false);
  const [triggerRef, setTriggerRef] = createSignal<HTMLElement>();
  const [tooltipRef, setTooltipRef] = createSignal<HTMLDivElement>();

  // Position state
  const [position, setPosition] = createSignal({ top: '0px', left: '0px' });

  let timeoutId: ReturnType<typeof setTimeout> | undefined;

  const clearTimer = () => {
    if (timeoutId !== undefined) {
      clearTimeout(timeoutId);
      timeoutId = undefined;
    }
  };

  const startTimer = () => {
    clearTimer();
    if (delay() > 0) {
      timeoutId = setTimeout(() => setIsOpen(true), delay());
    } else {
      setIsOpen(true);
    }
  };

  const show = () => setIsOpen(true);
  const hide = () => {
    clearTimer();
    setIsOpen(false);
  };

  const toggle = () => {
    if (isOpen()) {
      hide();
    } else {
      show();
    }
  };

  // Calculate position based on trigger and viewport
  const updatePosition = () => {
    const trigger = triggerRef();
    const tooltip = tooltipRef();
    if (!trigger || !tooltip) return;

    const triggerRect = trigger.getBoundingClientRect();
    const tooltipRect = tooltip.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    // Default: above trigger, centered
    let top = triggerRect.top - tooltipRect.height - 8;
    let left = triggerRect.left + triggerRect.width / 2 - tooltipRect.width / 2;

    // Flip to bottom if not enough space above
    if (top < 8 && triggerRect.bottom + 8 + tooltipRect.height <= viewportHeight) {
      top = triggerRect.bottom + 8;
    }

    // Keep within horizontal bounds
    if (left < 8) left = 8;
    if (left + tooltipRect.width > viewportWidth - 8) {
      left = viewportWidth - 8 - tooltipRect.width;
    }

    setPosition({ top: `${top}px`, left: `${left}px` });
  };

  // Update position when open changes or on scroll/resize
  createEffect(() => {
    if (!isOpen()) return;
    // Wait for DOM to update
    queueMicrotask(updatePosition);
  });

  createEffect(() => {
    if (!isOpen()) return;
    const handleUpdate = () => updatePosition();
    window.addEventListener('scroll', handleUpdate, true);
    window.addEventListener('resize', handleUpdate);
    onCleanup(() => {
      window.removeEventListener('scroll', handleUpdate, true);
      window.removeEventListener('resize', handleUpdate);
    });
  });

  // Keyboard: Escape closes, focus returns to trigger
  createEffect(() => {
    if (!isOpen()) return;
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        hide();
        triggerRef()?.focus();
      }
    };
    document.addEventListener('keydown', handleEscape);
    onCleanup(() => document.removeEventListener('keydown', handleEscape));
  });

  // Click outside handling
  createEffect(() => {
    if (!isOpen() || triggerMode() !== 'click') return;
    if (local.closeOnClickOutside === false) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (!tooltipRef()?.contains(e.target as Node) && !triggerRef()?.contains(e.target as Node)) {
        hide();
      }
    };
    document.addEventListener('click', handleClickOutside);
    onCleanup(() => document.removeEventListener('click', handleClickOutside));
  });

  // Hover handlers
  const handleMouseEnter = () => {
    if (triggerMode() === 'hover') startTimer();
  };

  const handleMouseLeave = () => {
    if (triggerMode() === 'hover') hide();
  };

  // Focus handlers for keyboard a11y (hover mode)
  const handleFocus = () => {
    if (triggerMode() === 'hover') show();
  };

  const handleBlur = () => {
    if (triggerMode() === 'hover') hide();
  };

  // Click handler for click mode
  const handleClick = (e: MouseEvent) => {
    if (triggerMode() === 'click') {
      e.preventDefault();
      e.stopPropagation();
      toggle();
    }
  };

  // Keyboard for click mode
  const handleKeyDown = (e: KeyboardEvent) => {
    if (triggerMode() === 'click' && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault();
      toggle();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      hide();
      triggerRef()?.focus();
    }
  };

  const tooltipId = createMemo(() => `omm-tooltip-${Math.random().toString(36).slice(2, 9)}`);

  return (
    <>
      <button
        type="button"
        ref={setTriggerRef}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        onFocus={handleFocus}
        onBlur={handleBlur}
        onClick={handleClick}
        onKeyDown={handleKeyDown}
        tabIndex={triggerMode() === 'click' ? 0 : undefined}
        aria-describedby={isOpen() ? tooltipId() : undefined}
        class={cx('omm-tooltip-trigger', local.class)}
        {...rest}
      >
        {local.children}
      </button>
      <Show when={isOpen()}>
        <div
          ref={setTooltipRef}
          id={tooltipId()}
          role="tooltip"
          class="omm-tooltip"
          style={position()}
        >
          {local.content}
        </div>
      </Show>
    </>
  );
}
