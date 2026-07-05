import { Alert, Button, Card, type Column, Spinner, Table, TextField } from '@omm/ui';
import type { JSX } from 'solid-js';
import { createSignal, Match, Show, Switch } from 'solid-js';
import { PriceHistoryChart } from '../components/PriceHistoryChart.tsx';
import type { Listing } from '../lib/auction.ts';
import { errorKey } from '../lib/errors.ts';
import { fmt, t } from '../lib/i18n.ts';
import { useBuyListing, useListings, usePriceHistory } from '../queries/useAuction.ts';

/** Auction browser — search + listings table + price history + buy intent. */
export default function AuctionHouse(): JSX.Element {
  const [query, setQuery] = createSignal('');
  const [selected, setSelected] = createSignal('');
  const listings = useListings(query);
  const history = usePriceHistory(selected);
  const buy = useBuyListing();

  const purchase = (listing: Listing): void => {
    setSelected(listing.item);
    buy.mutate({ listingId: listing.id, idempotencyKey: crypto.randomUUID() });
  };

  const columns: Column<Listing>[] = [
    {
      key: 'item',
      header: t('auction.col.item'),
      cell: (r) => (
        <button type="button" class="link-button text-accent" onClick={() => setSelected(r.item)}>
          {r.item}
        </button>
      ),
    },
    { key: 'qty', header: t('auction.col.quantity'), cell: (r) => fmt.integer(r.quantity) },
    {
      key: 'buyout',
      header: t('auction.col.buyout'),
      cell: (r) => `${fmt.gold(r.buyoutPer)} ${t('auction.each')}`,
    },
    { key: 'seller', header: t('auction.col.seller'), cell: (r) => r.seller },
    {
      key: 'ends',
      header: t('auction.col.time'),
      cell: (r) => fmt.dateTime(new Date(r.endsAt)),
    },
    {
      key: 'buy',
      header: '',
      cell: (r) => (
        <Button variant="primary" disabled={buy.isPending} onClick={() => purchase(r)}>
          {buy.isPending ? t('auction.buying') : t('auction.buy')}
        </Button>
      ),
    },
  ];

  return (
    <div class="stack">
      <Card title={t('auction.heading')} class="stack">
        <Show when={buy.isSuccess}>
          <Alert tone="success">
            {t('auction.bought', {
              item: buy.data?.item ?? '',
              price: fmt.gold(buy.data?.price ?? 0),
            })}
          </Alert>
        </Show>
        <Show when={buy.isError}>
          <Alert tone="error">{t(errorKey(buy.error))}</Alert>
        </Show>
        <div class="toolbar">
          <TextField
            id="ah-search"
            label={t('common.search')}
            placeholder={t('auction.searchPlaceholder')}
            value={query()}
            onInput={(e) => setQuery(e.currentTarget.value)}
          />
        </div>
        <Switch>
          <Match when={listings.isPending}>
            <Spinner label={t('common.loading')} />
          </Match>
          <Match when={listings.isError}>
            <Alert tone="error">{t(errorKey(listings.error))}</Alert>
          </Match>
          <Match when={listings.data}>
            {(rows) => (
              <Show
                when={rows().length > 0}
                fallback={<p class="text-fg-muted">{t('auction.empty')}</p>}
              >
                <Table columns={columns} rows={rows()} rowKey={(r) => r.id} />
              </Show>
            )}
          </Match>
        </Switch>
      </Card>

      <Show when={selected()}>
        <Card title={`${t('auction.priceHistory')} — ${selected()}`}>
          <Show when={history.data} fallback={<Spinner label={t('common.loading')} />}>
            {(h) => <PriceHistoryChart points={h().points} />}
          </Show>
        </Card>
      </Show>
    </div>
  );
}
