import type { JSX } from 'solid-js';
import { cx } from './cx.ts';

/** Severity of the alert — selects the accent border token. */
export type AlertTone = 'error' | 'info' | 'success';

export interface AlertProps {
  tone?: AlertTone;
  class?: string;
  children: JSX.Element;
}

/** An inline status banner. `role` is set for assistive tech by tone. */
export function Alert(props: AlertProps): JSX.Element {
  const tone = (): AlertTone => props.tone ?? 'info';
  return (
    <div
      class={cx('alert', `alert--${tone()}`, props.class)}
      role={tone() === 'error' ? 'alert' : 'status'}
    >
      {props.children}
    </div>
  );
}
