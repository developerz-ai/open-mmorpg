import {
  Alert,
  Button,
  Card,
  type Column,
  Select,
  SelectOption,
  Spinner,
  Table,
  TextField,
} from '@omm/ui';
import type { JSX } from 'solid-js';
import { createEffect, createMemo, createSignal, Match, Show, Switch } from 'solid-js';
import { PriceHistoryChart } from '../components/PriceHistoryChart.tsx';
import type { Listing } from '../lib/auction.ts';
import { errorKey } from '../lib/errors.ts';
import { fmt, t } from '../lib/i18n.ts';
import { useBuyListing, useListings, usePriceHistory } from '../queries/useAuction.ts';

/** Item categories for filtering */
const ITEM_CATEGORIES = ['Weapon', 'Armor', 'Consumable', 'Material', 'Misc'] as const;
type ItemCategory = (typeof ITEM_CATEGORIES)[number];

/** Sort options */
const SORT_OPTIONS = ['priceAsc', 'priceDesc', 'quantity', 'timeRemaining'] as const;
type SortOption = (typeof SORT_OPTIONS)[number];

const ITEMS_PER_PAGE = 20;

/** Auction browser — search + filters + sorting + pagination + listings table + price history + buy intent. */
export default function AuctionHouse(): JSX.Element {
  const [query, setQuery] = createSignal('');
  const [category, setCategory] = createSignal<ItemCategory | undefined>();
  const [minPrice, setMinPrice] = createSignal<number | undefined>();
  const [maxPrice, setMaxPrice] = createSignal<number | undefined>();
  const [sort, setSort] = createSignal<SortOption>('priceAsc');
  const [page, setPage] = createSignal(1);
  const [selected, setSelected] = createSignal('');

  const listings = useListings(query);
  const history = usePriceHistory(selected);
  const buy = useBuyListing();

  // Reset page on filter/sort change
  createEffect(() => {
    setPage(1);
  });

  // Filter, sort, and paginate listings
  const processedListings = createMemo(() => {
    if (listings.data === undefined) return [];

    let filtered = [...listings.data];

    // Category filter
    const cat = category();
    if (cat) {
      filtered = filtered.filter((l) => l.item.startsWith(cat));
    }

    // Price range filter
    const min = minPrice();
    const max = maxPrice();
    if (min !== undefined) {
      filtered = filtered.filter((l) => l.buyoutPer >= min);
    }
    if (max !== undefined) {
      filtered = filtered.filter((l) => l.buyoutPer <= max);
    }

    // Sorting
    switch (sort()) {
      case 'priceAsc':
        filtered.sort((a, b) => a.buyoutPer - b.buyoutPer);
        break;
      case 'priceDesc':
        filtered.sort((a, b) => b.buyoutPer - a.buyoutPer);
        break;
      case 'quantity':
        filtered.sort((a, b) => b.quantity - a.quantity);
        break;
      case 'timeRemaining':
        filtered.sort((a, b) => new Date(a.endsAt).getTime() - new Date(b.endsAt).getTime());
        break;
    }

    return filtered;
  });

  // Pagination
  const totalPages = createMemo(() => Math.ceil(processedListings().length / ITEMS_PER_PAGE) || 1);
  const paginatedListings = createMemo(() => {
    const start = (page() - 1) * ITEMS_PER_PAGE;
    return processedListings().slice(start, start + ITEMS_PER_PAGE);
  });

  const paginationInfo = createMemo(() => {
    const total = processedListings().length;
    const start = total === 0 ? 0 : (page() - 1) * ITEMS_PER_PAGE + 1;
    const end = Math.min(page() * ITEMS_PER_PAGE, total);
    return { start, end, total };
  });

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

        {/* Search and filters */}
        <div class="toolbar">
          <TextField
            id="ah-search"
            label={t('common.search')}
            placeholder={t('auction.searchPlaceholder')}
            value={query()}
            onInput={(e) => setQuery(e.currentTarget.value)}
          />

          <Select
            id="ah-category"
            label={t('auction.filter.category')}
            value={category()}
            onChange={(v) => setCategory(v as ItemCategory | undefined)}
            placeholder={t('auction.filter.allCategories')}
          >
            <SelectOption value="">{t('auction.filter.allCategories')}</SelectOption>
            {ITEM_CATEGORIES.map((cat) => (
              <SelectOption value={cat}>{cat}</SelectOption>
            ))}
          </Select>

          <div class="filter-row">
            <TextField
              id="ah-min-price"
              label={t('auction.filter.minPrice')}
              type="number"
              min="0"
              value={minPrice() ?? ''}
              onInput={(e) =>
                setMinPrice(e.currentTarget.value ? Number(e.currentTarget.value) : undefined)
              }
            />
            <TextField
              id="ah-max-price"
              label={t('auction.filter.maxPrice')}
              type="number"
              min="0"
              value={maxPrice() ?? ''}
              onInput={(e) =>
                setMaxPrice(e.currentTarget.value ? Number(e.currentTarget.value) : undefined)
              }
            />
          </div>

          <Select
            id="ah-sort"
            label={t('auction.sort.label')}
            value={sort()}
            onChange={(v) => setSort(v as SortOption)}
          >
            <SelectOption value="priceAsc">{t('auction.sort.priceAsc')}</SelectOption>
            <SelectOption value="priceDesc">{t('auction.sort.priceDesc')}</SelectOption>
            <SelectOption value="quantity">{t('auction.sort.quantity')}</SelectOption>
            <SelectOption value="timeRemaining">{t('auction.sort.timeRemaining')}</SelectOption>
          </Select>
        </div>

        <Switch>
          <Match when={listings.isPending}>
            <Spinner label={t('common.loading')} />
          </Match>
          <Match when={listings.isError}>
            <Alert tone="error">{t(errorKey(listings.error))}</Alert>
          </Match>
          <Match when={listings.data}>
            <Show
              when={paginatedListings().length > 0}
              fallback={<p class="text-fg-muted">{t('auction.empty')}</p>}
            >
              <Table columns={columns} rows={paginatedListings()} rowKey={(r) => r.id} />

              {/* Pagination controls */}
              <div class="pagination">
                <Button
                  variant="ghost"
                  disabled={page() === 1}
                  onClick={() => setPage((p) => Math.max(1, p - 1))}
                >
                  {t('auction.pagination.prev')}
                </Button>
                <span class="pagination-info">
                  {t('auction.pagination.page', { current: page(), total: totalPages() })}
                </span>
                <Button
                  variant="ghost"
                  disabled={page() === totalPages()}
                  onClick={() => setPage((p) => Math.min(totalPages(), p + 1))}
                >
                  {t('auction.pagination.next')}
                </Button>
                <span class="pagination-info text-fg-muted">
                  {t('auction.pagination.showing', paginationInfo())}
                </span>
              </div>
            </Show>
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
