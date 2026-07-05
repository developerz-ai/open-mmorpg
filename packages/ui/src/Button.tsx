import type { JSX } from 'solid-js';
import { splitProps } from 'solid-js';
import { cx } from './cx.ts';

/** Visual weight of the button. Roles map to semantic theme tokens. */
export type ButtonVariant = 'primary' | 'ghost';

export interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
}

/**
 * The one shared button. SRP: styling by role only, no raw colors. Consumers
 * pass through native button attributes; `variant` selects the token set.
 */
export function Button(props: ButtonProps): JSX.Element {
  const [local, rest] = splitProps(props, ['variant', 'class', 'children']);
  const variant = (): ButtonVariant => local.variant ?? 'primary';
  return (
    <button type="button" class={cx('omm-btn', `omm-btn--${variant()}`, local.class)} {...rest}>
      {local.children}
    </button>
  );
}
