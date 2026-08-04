import type { JSX } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { Show } from 'solid-js';
import { cx } from './cx.ts';

export interface CardProps {
  /** Optional heading rendered above the body. Caller passes a `t()`'d string. */
  title?: JSX.Element;
  class?: string;
  /**
   * Heading level for the card title. Defaults to 2. A route whose primary
   * card IS the page heading passes 1 — six routes had no `<h1>` at all
   * because every card title was an h2.
   */
  headingLevel?: 1 | 2;
  children: JSX.Element;
}

/** A raised surface panel. Presentational only — no data, no strings of its own. */
export function Card(props: CardProps): JSX.Element {
  return (
    <section class={cx('card', props.class)}>
      <Show when={props.title}>
        <Dynamic component={props.headingLevel === 1 ? 'h1' : 'h2'} class="card__title">
          {props.title}
        </Dynamic>
      </Show>
      {props.children}
    </section>
  );
}
