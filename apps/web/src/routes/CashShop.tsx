import { Alert, Button, Card, type Column, Spinner, Table } from '@omm/ui';
import type { JSX } from 'solid-js';
import { createSignal, Match, Show, Switch } from 'solid-js';
import type { ShopItem } from '../lib/cashShop.ts';
import { errorKey } from '../lib/errors.ts';
import { t } from '../lib/i18n.ts';
import { useBuyShopItem, useShopCategories, useShopItems } from '../queries/useCashShop.ts';

/** Cash shop — featured items, categories, currency display, and purchase. */
export default function CashShop(): JSX.Element {
  const [category, setCategory] = createSignal<string>('');
  const items = useShopItems(category);
  const categories = useShopCategories();
  const buy = useBuyShopItem();

  const purchase = (itemId: string): void => {
    buy.mutate({ itemId, idempotencyKey: crypto.randomUUID() });
  };

  const columns: Column<ShopItem>[] = [
    {
      key: 'name',
      header: t('cashShop.col.item'),
      cell: (r) => (
        <div class="stack-sm">
          <div class="text-fg">{r.name}</div>
          <Show when={r.description}>
            <div class="text-fg-muted text-sm">{r.description}</div>
          </Show>
        </div>
      ),
    },
    {
      key: 'category',
      header: t('cashShop.col.category'),
      cell: (r) => t(`cashShop.category.${r.category}`),
    },
    {
      key: 'price',
      header: t('cashShop.col.price'),
      cell: (r) => `${r.price} ${t('cashShop.currency')}`,
    },
    {
      key: 'buy',
      header: '',
      cell: (r) => (
        <Button variant="primary" disabled={buy.isPending} onClick={() => purchase(r.id)}>
          {buy.isPending ? t('cashShop.buying') : t('cashShop.buy')}
        </Button>
      ),
    },
  ];

  return (
    <div class="stack">
      <Card title={t('cashShop.heading')} class="stack">
        <Show when={buy.isSuccess}>
          <Alert tone="success">
            {t('cashShop.bought', {
              item: buy.data?.item ?? '',
              price: buy.data?.price ?? 0,
            })}
          </Alert>
        </Show>
        <Show when={buy.isError}>
          <Alert tone="error">{t(errorKey(buy.error))}</Alert>
        </Show>

        {/* Categories */}
        <Show when={categories.data} fallback={<Spinner label={t('common.loading')} />}>
          {(cats) => (
            <div class="toolbar">
              <button
                type="button"
                class="chip"
                classList={{ active: category() === '' }}
                onClick={() => setCategory('')}
              >
                {t('cashShop.allCategories')}
              </button>
              {cats().map((c) => (
                <button
                  type="button"
                  class="chip"
                  classList={{ active: category() === c }}
                  onClick={() => setCategory(c)}
                >
                  {t(`cashShop.category.${c}`)}
                </button>
              ))}
            </div>
          )}
        </Show>

        {/* Items */}
        <Switch>
          <Match when={items.isPending}>
            <Spinner label={t('common.loading')} />
          </Match>
          <Match when={items.isError}>
            <Alert tone="error">{t(errorKey(items.error))}</Alert>
          </Match>
          <Match when={items.data}>
            {(rows) => (
              <Show
                when={rows().length > 0}
                fallback={<p class="text-fg-muted">{t('cashShop.empty')}</p>}
              >
                <Table columns={columns} rows={rows()} rowKey={(r) => r.id} />
              </Show>
            )}
          </Match>
        </Switch>
      </Card>
    </div>
  );
}
