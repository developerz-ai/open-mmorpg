import type { JSX } from 'solid-js';
import { splitProps } from 'solid-js';
import { cx } from './cx.ts';

export type SpinnerSize = 'sm' | 'md' | 'lg';

export interface SpinnerProps {
  /** Accessible label (a `t()`'d string) — required so the spinner isn't silent. */
  label: string;
  /** Size variant. Defaults to `md`. */
  size?: SpinnerSize;
  class?: string;
}

/** A loading indicator. The label is SR-only; the ring honors reduced-motion. */
export function Spinner(props: SpinnerProps): JSX.Element {
  const [local, rest] = splitProps(props, ['label', 'size', 'class']);
  const size = (): SpinnerSize => local.size ?? 'md';

  return (
    <span
      class={cx('spinner', `spinner--${size()}`, local.class)}
      role="status"
      aria-live="polite"
      {...rest}
    >
      <span class="sr-only">{local.label}</span>
    </span>
  );
}
