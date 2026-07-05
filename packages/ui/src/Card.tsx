import type { JSX } from 'solid-js';
import { Show } from 'solid-js';
import { cx } from './cx.ts';

export interface CardProps {
  /** Optional heading rendered above the body. Caller passes a `t()`'d string. */
  title?: JSX.Element;
  class?: string;
  children: JSX.Element;
}

/** A raised surface panel. Presentational only — no data, no strings of its own. */
export function Card(props: CardProps): JSX.Element {
  return (
    <section class={cx('card', props.class)}>
      <Show when={props.title}>
        <h2 class="card__title">{props.title}</h2>
      </Show>
      {props.children}
    </section>
  );
}
