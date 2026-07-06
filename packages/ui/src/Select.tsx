import type { JSX } from 'solid-js';
import {
  createContext,
  createEffect,
  createMemo,
  createSignal,
  Show,
  splitProps,
  useContext,
} from 'solid-js';
import { cx } from './cx.ts';

interface SelectContextValue {
  value: () => string | undefined;
  select: (value: string) => void;
  close: () => void;
  registerOption: (value: string, element: HTMLDivElement) => () => void;
  focusNext: (direction: 1 | -1) => void;
}

const SelectContext = createContext<SelectContextValue>();

function useSelectContext(): SelectContextValue {
  const ctx = useContext(SelectContext);
  if (!ctx) throw new Error('SelectOption must be used inside Select');
  return ctx;
}

export interface SelectProps extends Omit<JSX.HTMLAttributes<HTMLDivElement>, 'onChange'> {
  /** Currently selected value. */
  value: string | undefined;
  /** Fired when selection changes. */
  onChange: (value: string) => void;
  /** Field id — ties the label to the select for a11y. */
  id: string;
  /** Visible label text (a `t()`'d string from the caller). */
  label: string;
  /** Optional placeholder when no value is selected. */
  placeholder?: string;
  /** Optional validation message; presence sets `aria-invalid`. */
  error?: string;
}

/**
 * A dropdown select with keyboard navigation. Arrows move focus, Enter selects,
 * Escape closes. Controlled component — parent owns the selected value.
 */
export function Select(props: SelectProps) {
  const [local, rest] = splitProps(props, [
    'value',
    'onChange',
    'id',
    'label',
    'placeholder',
    'error',
    'class',
    'children',
  ]);
  const errorId = (): string => `${local.id}-error`;
  const listboxId = (): string => `${local.id}-listbox`;

  const [isOpen, setIsOpen] = createSignal(false);
  const [triggerRef, setTriggerRef] = createSignal<HTMLDivElement>();
  const [highlightedIndex, setHighlightedIndex] = createSignal<number>(-1);

  const options: { value: string; element: HTMLDivElement }[] = [];

  const registerOption = (value: string, element: HTMLDivElement) => {
    options.push({ value, element });
    return () => {
      const idx = options.findIndex((o) => o.value === value);
      if (idx >= 0) options.splice(idx, 1);
    };
  };

  const focusNext = (direction: 1 | -1) => {
    if (options.length === 0) return;
    const current = highlightedIndex();
    let next = current + direction;
    if (next < 0) next = options.length - 1;
    if (next >= options.length) next = 0;
    setHighlightedIndex(next);
    options[next]?.element.focus();
  };

  const select = (value: string) => {
    local.onChange(value);
    setIsOpen(false);
    setHighlightedIndex(-1);
    triggerRef()?.focus();
  };

  const close = () => {
    setIsOpen(false);
    setHighlightedIndex(-1);
    triggerRef()?.focus();
  };

  const contextValue: SelectContextValue = {
    value: () => local.value,
    select,
    close,
    registerOption,
    focusNext,
  };

  const displayLabel = createMemo(() => {
    if (!local.value) return local.placeholder ?? 'Select...';
    const option = options.find((o) => o.value === local.value);
    return option?.element.textContent?.trim() ?? local.placeholder ?? 'Select...';
  });

  const handleTriggerClick = () => {
    setIsOpen(!isOpen());
    if (!isOpen()) {
      // Focus first option on open
      queueMicrotask(() => {
        options[0]?.element.focus();
        setHighlightedIndex(0);
      });
    }
  };

  const handleTriggerKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      setIsOpen(!isOpen());
    } else if (e.key === 'ArrowDown' && isOpen()) {
      e.preventDefault();
      focusNext(1);
    } else if (e.key === 'ArrowUp' && isOpen()) {
      e.preventDefault();
      focusNext(-1);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      close();
    }
  };

  createEffect(() => {
    if (!isOpen()) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (!triggerRef()?.contains(e.target as Node)) {
        close();
      }
    };
    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  });

  return (
    <div class={cx('field', local.class)} {...rest}>
      <label class="field__label" for={local.id}>
        {local.label}
      </label>
      <div
        ref={setTriggerRef}
        id={local.id}
        role="combobox"
        tabIndex={0}
        aria-haspopup="listbox"
        aria-expanded={isOpen() ? 'true' : 'false'}
        aria-controls={listboxId()}
        aria-invalid={local.error ? 'true' : undefined}
        aria-describedby={local.error ? errorId() : undefined}
        class={cx('select-trigger', local.error && 'select-trigger--invalid')}
        onClick={handleTriggerClick}
        onKeyDown={handleTriggerKeyDown}
      >
        <span class="select-trigger__label">{displayLabel()}</span>
        <span class="select-trigger__icon" aria-hidden="true">
          ▼
        </span>
      </div>
      <SelectContext.Provider value={contextValue}>
        <Show when={isOpen()}>
          <div id={listboxId()} role="listbox" class="select-listbox">
            {local.children}
          </div>
        </Show>
      </SelectContext.Provider>
      <Show when={local.error}>
        <span id={errorId()} class="field__error">
          {local.error}
        </span>
      </Show>
    </div>
  );
}

export interface SelectOptionProps extends JSX.HTMLAttributes<HTMLDivElement> {
  /** Option value — passed back to onChange. */
  value: string;
}

/**
 * A single option inside Select. Registers itself with parent, receives
 * keyboard focus, handles click/Enter selection.
 */
export function SelectOption(props: SelectOptionProps) {
  const [local, rest] = splitProps(props, ['value', 'class', 'children']);
  const ctx = useSelectContext();
  let elementRef: HTMLDivElement | undefined;

  createEffect(() => {
    if (elementRef) {
      const unregister = ctx.registerOption(local.value, elementRef);
      return unregister;
    }
  });

  const isSelected = () => ctx.value() === local.value;

  const handleClick = () => ctx.select(local.value);

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      ctx.select(local.value);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      ctx.close();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      ctx.focusNext(1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      ctx.focusNext(-1);
    }
  };

  return (
    <div
      ref={elementRef}
      role="option"
      aria-selected={isSelected() ? 'true' : 'false'}
      tabIndex={-1}
      class={cx('select-option', isSelected() && 'select-option--selected', local.class)}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      {...rest}
    >
      {local.children}
    </div>
  );
}
