import type { JSX } from 'solid-js';
import { splitProps } from 'solid-js';
import { cx } from './cx.ts';

export interface ProgressProps extends JSX.HTMLAttributes<HTMLDivElement> {
  /** Percentage complete (0-100). Defaults to 0. */
  value?: number;
  /** Accessible label describing what's being loaded. */
  label: string;
}

/**
 * Linear progress bar. Renders a determinate bar when `value` is set;
 * otherwise shows an indeterminate loading state. The label is SR-only.
 */
export function Progress(props: ProgressProps): JSX.Element {
  const [local, rest] = splitProps(props, ['value', 'label', 'class']);
  const percent = () => Math.min(100, Math.max(0, local.value ?? 0));

  return (
    <div
      class={cx('omm-progress', local.class)}
      role="progressbar"
      aria-valuenow={local.value ?? undefined}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label={local.label}
      {...rest}
    >
      <div class="omm-progress__bar" style={`width: ${percent()}%`} />
    </div>
  );
}
