import type { JSX } from 'solid-js';
import { createContext, createEffect, createMemo, Show, splitProps, useContext } from 'solid-js';
import { cx } from './cx.ts';

interface TabsContextValue {
  value: () => string | undefined;
  select: (value: string) => void;
  registerTab: (value: string, element: HTMLButtonElement) => () => void;
  focusNext: (direction: 1 | -1) => void;
  focusFirst: () => void;
  focusLast: () => void;
}

const TabsContext = createContext<TabsContextValue>();

function useTabsContext(): TabsContextValue {
  const ctx = useContext(TabsContext);
  if (!ctx) throw new Error('Tab compound components must be used inside Tabs');
  return ctx;
}

export interface TabsProps extends Omit<JSX.HTMLAttributes<HTMLDivElement>, 'onChange'> {
  /** Currently selected tab value. */
  value: string | undefined;
  /** Fired when tab selection changes. */
  onChange: (value: string) => void;
}

/**
 * Tab container with keyboard navigation. Compound component: use TabList,
 * Tab, and TabPanel as children. Controlled component — parent owns selected
 * value via value/onChange.
 *
 * Keyboard: Arrow Left/Right move between tabs, Home/First jump to ends,
 * Enter/Space selects.
 */
export function Tabs(props: TabsProps) {
  const [local, rest] = splitProps(props, ['value', 'onChange', 'class', 'children']);

  const tabs: { value: string; element: HTMLButtonElement }[] = [];

  const registerTab = (value: string, element: HTMLButtonElement) => {
    tabs.push({ value, element });
    return () => {
      const idx = tabs.findIndex((t) => t.value === value);
      if (idx >= 0) tabs.splice(idx, 1);
    };
  };

  const focusNext = (direction: 1 | -1) => {
    if (tabs.length === 0) return;
    const currentIdx = tabs.findIndex((t) => t.value === local.value);
    let nextIdx = currentIdx + direction;
    if (nextIdx < 0) nextIdx = tabs.length - 1;
    if (nextIdx >= tabs.length) nextIdx = 0;
    const nextTab = tabs[nextIdx];
    if (nextTab) {
      nextTab.element.focus();
      local.onChange(nextTab.value);
    }
  };

  const focusFirst = () => {
    if (tabs.length === 0) return;
    const first = tabs[0];
    if (first) {
      first.element.focus();
      local.onChange(first.value);
    }
  };

  const focusLast = () => {
    if (tabs.length === 0) return;
    const last = tabs[tabs.length - 1];
    if (last) {
      last.element.focus();
      local.onChange(last.value);
    }
  };

  const contextValue: TabsContextValue = {
    value: () => local.value,
    select: (value: string) => local.onChange(value),
    registerTab,
    focusNext,
    focusFirst,
    focusLast,
  };

  return (
    <TabsContext.Provider value={contextValue}>
      <div class={cx('tabs', local.class)} {...rest}>
        {local.children}
      </div>
    </TabsContext.Provider>
  );
}

export interface TabListProps extends JSX.HTMLAttributes<HTMLDivElement> {}

/**
 * Container for tab buttons. Renders with role="tablist" for accessibility.
 */
export function TabList(props: TabListProps) {
  const [local, rest] = splitProps(props, ['class', 'children']);
  return (
    <div role="tablist" class={cx('tab-list', local.class)} {...rest}>
      {local.children}
    </div>
  );
}

export interface TabProps extends Omit<JSX.HTMLAttributes<HTMLButtonElement>, 'value'> {
  /** Unique tab value — passed to onChange when selected. */
  value: string;
}

/**
 * Individual tab button. Registers with parent, handles keyboard navigation
 * and selection, renders with full ARIA attributes.
 */
export function Tab(props: TabProps) {
  const [local, rest] = splitProps(props, ['value', 'class', 'children']);
  const ctx = useTabsContext();
  let elementRef: HTMLButtonElement | undefined;

  createEffect(() => {
    if (elementRef) {
      const unregister = ctx.registerTab(local.value, elementRef);
      return unregister;
    }
  });

  const isSelected = createMemo(() => ctx.value() === local.value);

  const handleClick = () => ctx.select(local.value);

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      ctx.select(local.value);
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      ctx.focusNext(1);
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      ctx.focusNext(-1);
    } else if (e.key === 'Home') {
      e.preventDefault();
      ctx.focusFirst();
    } else if (e.key === 'End') {
      e.preventDefault();
      ctx.focusLast();
    }
  };

  const tabId = () => `tab-${local.value}`;
  const panelId = () => `tabpanel-${local.value}`;

  return (
    <button
      ref={elementRef}
      type="button"
      role="tab"
      id={tabId()}
      aria-selected={isSelected() ? 'true' : 'false'}
      aria-controls={panelId()}
      tabIndex={isSelected() ? 0 : -1}
      class={cx('tab', isSelected() && 'tab--selected', local.class)}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      {...rest}
    >
      {local.children}
    </button>
  );
}

export interface TabPanelProps extends JSX.HTMLAttributes<HTMLDivElement> {
  /** Tab value — must match a Tab's value for accessibility linkage. */
  value: string;
}

/**
 * Content panel for a tab. Only visible when its tab is selected. Renders
 * with role="tabpanel" and proper ARIA linkage to its tab.
 */
export function TabPanel(props: TabPanelProps) {
  const [local, rest] = splitProps(props, ['value', 'class', 'children']);
  const ctx = useTabsContext();

  const isSelected = createMemo(() => ctx.value() === local.value);

  const tabId = () => `tab-${local.value}`;
  const panelId = () => `tabpanel-${local.value}`;

  return (
    <Show when={isSelected()}>
      <div
        role="tabpanel"
        id={panelId()}
        aria-labelledby={tabId()}
        class={cx('tab-panel', local.class)}
        {...rest}
      >
        {local.children}
      </div>
    </Show>
  );
}
