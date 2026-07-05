import type { JSX } from 'solid-js';
import { For, Show } from 'solid-js';
import type { PricePoint } from '../lib/auction.ts';
import { fmt, t } from '../lib/i18n.ts';

/**
 * A tiny token-only bar chart of average buyout over time. Presentational: it
 * takes already-fetched points and renders bars scaled to the max. Colors come
 * from theme tokens (accent), never raw hex. → docs/specs/web-client/design-system
 */
export function PriceHistoryChart(props: { points: PricePoint[] }): JSX.Element {
  const max = (): number => Math.max(1, ...props.points.map((p) => p.avgBuyout));
  return (
    <Show
      when={props.points.length > 0}
      fallback={<p class="text-fg-muted">{t('common.empty')}</p>}
    >
      <div class="chart" role="img" aria-label={t('auction.priceHistory')}>
        <For each={props.points}>
          {(p) => (
            <div
              class="chart__col"
              title={`${fmt.gold(p.avgBuyout)} — ${fmt.date(new Date(p.at))}`}
            >
              <div class="chart__bar" style={{ height: `${(p.avgBuyout / max()) * 100}%` }} />
              <span class="chart__label text-fg-muted">{fmt.compact(p.avgBuyout)}</span>
            </div>
          )}
        </For>
      </div>
    </Show>
  );
}
