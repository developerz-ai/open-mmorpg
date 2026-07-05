import type { JSX } from 'solid-js';
import { Show, splitProps } from 'solid-js';
import { cx } from './cx.ts';

export interface TextFieldProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
  /** Field id — ties the label to the input for a11y. */
  id: string;
  /** Visible label text (a `t()`'d string from the caller). */
  label: string;
  /** Optional validation message; presence sets `aria-invalid`. */
  error?: string;
}

/**
 * A labelled text input. Presentational + a11y wiring only: the label is bound
 * to the input, the error is announced, and the value/handlers pass through.
 */
export function TextField(props: TextFieldProps): JSX.Element {
  const [local, rest] = splitProps(props, ['id', 'label', 'error', 'class']);
  const errorId = (): string => `${local.id}-error`;
  return (
    <div class={cx('field', local.class)}>
      <label class="field__label" for={local.id}>
        {local.label}
      </label>
      <input
        id={local.id}
        class="input"
        aria-invalid={local.error ? 'true' : undefined}
        aria-describedby={local.error ? errorId() : undefined}
        {...rest}
      />
      <Show when={local.error}>
        <span id={errorId()} class="field__error">
          {local.error}
        </span>
      </Show>
    </div>
  );
}
