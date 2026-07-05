import type { JSX } from 'solid-js';
import { cx } from './cx.ts';

export interface SpinnerProps {
  /** Accessible label (a `t()`'d string) — required so the spinner isn't silent. */
  label: string;
  class?: string;
}

/** A loading indicator. The label is SR-only; the ring honors reduced-motion. */
export function Spinner(props: SpinnerProps): JSX.Element {
  return (
    <span class={cx('spinner', props.class)} role="status" aria-live="polite">
      <span class="sr-only">{props.label}</span>
    </span>
  );
}
