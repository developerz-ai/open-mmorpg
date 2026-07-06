import { Alert, Card, Spinner } from '@omm/ui';
import type { JSX } from 'solid-js';
import { For, Match, Show, Switch } from 'solid-js';
import type { ShopItem } from '../lib/cashShop.ts';
import { errorKey } from '../lib/errors.ts';
import { t } from '../lib/i18n.ts';
import { ShopItem as ShopItemComponent } from './ShopItem.tsx';

export interface ShopGridProps {
  items: ShopItem[];
  categories: string[];
  isPending: boolean;
  isError: boolean;
  error: Error | null;
  isPurchasePending: boolean;
  onPurchase: (itemId: string) => void;
  onCategoryChange: (category: string) => void;
  selectedCategory: string;
}

/**
 * Shop grid — card-based layout with category filter chips. Displays
 * items in a responsive grid with purchase handlers. SRP: presentation
 * and filtering, no data fetching.
 */
export function ShopGrid(props: ShopGridProps): JSX.Element {
  return (
    <Card title={t('cashShop.heading')} class="shop-grid">
      {/* Categories */}
      <Show when={props.categories.length > 0} fallback={<Spinner label={t('common.loading')} />}>
        <div class="shop-grid__filters">
          <button
            type="button"
            class="chip"
            classList={{ active: props.selectedCategory === '' }}
            onClick={() => props.onCategoryChange('')}
          >
            {t('cashShop.allCategories')}
          </button>
          <For each={props.categories}>
            {(category) => (
              <button
                type="button"
                class="chip"
                classList={{ active: props.selectedCategory === category }}
                onClick={() => props.onCategoryChange(category)}
              >
                {t(`cashShop.category.${category}`)}
              </button>
            )}
          </For>
        </div>
      </Show>

      {/* Items */}
      <Switch>
        <Match when={props.isPending}>
          <div class="shop-grid__loading">
            <Spinner label={t('common.loading')} />
          </div>
        </Match>
        <Match when={props.isError}>
          <Alert tone="error">{t(errorKey(props.error))}</Alert>
        </Match>
        <Match when={props.items}>
          {(rows) => (
            <Show
              when={rows().length > 0}
              fallback={<p class="text-fg-muted shop-grid__empty">{t('cashShop.empty')}</p>}
            >
              <div class="shop-grid__items">
                <For each={rows()}>
                  {(item) => (
                    <ShopItemComponent
                      item={item}
                      isPending={props.isPurchasePending}
                      onPurchase={props.onPurchase}
                    />
                  )}
                </For>
              </div>
            </Show>
          )}
        </Match>
      </Switch>
    </Card>
  );
}
