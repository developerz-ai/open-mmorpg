import type { JSX } from 'solid-js';
import { cx } from './cx.ts';

/** Semantic weight of the badge — maps to a token set, never a raw color. */
export type BadgeTone = 'accent' | 'neutral' | 'success' | 'danger' | 'warning';

export interface BadgeProps {
  tone?: BadgeTone;
  class?: string;
  children: JSX.Element;
}

/** A small pill for status/labels. Tone selects the token set. */
export function Badge(props: BadgeProps): JSX.Element {
  const tone = (): BadgeTone => props.tone ?? 'accent';
  return (
    <span class={cx('badge', tone() !== 'accent' && `badge--${tone()}`, props.class)}>
      {props.children}
    </span>
  );
}
