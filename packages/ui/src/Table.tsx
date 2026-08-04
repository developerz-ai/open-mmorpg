import type { JSX } from 'solid-js';
import { For } from 'solid-js';
import { cx } from './cx.ts';

/** One column: a header cell and a per-row cell renderer. */
export interface Column<T> {
  /** Stable key for the column (also the header cell's list key). */
  key: string;
  /** Header content — a `t()`'d string from the caller. */
  header: JSX.Element;
  /** Render this column's cell for a row. */
  cell: (row: T) => JSX.Element;
}

export interface TableProps<T> {
  /** Accessible name for the scroll region when the table is wider than the viewport. */
  scrollLabel?: string;
  columns: Column<T>[];
  rows: T[];
  /** Stable key extractor so rows reconcile correctly. */
  rowKey: (row: T) => string | number;
  class?: string;
}

/** A presentational data table. No fetching, no sorting logic — pure render. */
export function Table<T>(props: TableProps<T>): JSX.Element {
  return (
    /* Wrapped so a table wider than the viewport scrolls INSIDE its own box.
     * Unwrapped, the auction listings pushed the whole page sideways on a
     * phone and every other element moved with it. `tabindex` + `role=region`
     * make that scroll reachable without a pointer (WCAG 2.1.1); the label
     * names the region a keyboard user has just landed in. */
    <div class="table-scroll" tabindex={0} role="region" aria-label={props.scrollLabel ?? 'Table'}>
    <table class={cx('table', props.class)}>
      <thead>
        <tr>
          <For each={props.columns}>{(col) => <th>{col.header}</th>}</For>
        </tr>
      </thead>
      <tbody>
        <For each={props.rows}>
          {(row) => (
            <tr data-row-key={props.rowKey(row)}>
              <For each={props.columns}>{(col) => <td>{col.cell(row)}</td>}</For>
            </tr>
          )}
        </For>
      </tbody>
    </table>
    </div>
  );
}
