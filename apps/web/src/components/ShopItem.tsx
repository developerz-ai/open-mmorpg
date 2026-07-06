import { Badge, Button, Card, Spinner } from '@omm/ui';
import type { JSX } from 'solid-js';
import { Show } from 'solid-js';
import type { ShopItem as ShopItemType } from '../lib/cashShop.ts';
import { t } from '../lib/i18n.ts';

export interface ShopItemProps {
  item: ShopItemType;
  isPending: boolean;
  onPurchase: (itemId: string) => void;
}

/**
 * Shop item card — displays item name, description, category badge,
 * price, and purchase button. SRP: presentation only, no data fetching.
 */
export function ShopItem(props: ShopItemProps): JSX.Element {
  const handlePurchase = (): void => {
    props.onPurchase(props.item.id);
  };

  return (
    <Card class="shop-item">
      <div class="shop-item__header">
        <h3 class="shop-item__name">{props.item.name}</h3>
        <Badge tone="accent">{t(`cashShop.category.${props.item.category}`)}</Badge>
      </div>
      <Show when={props.item.description}>
        <p class="shop-item__description text-fg-muted">{props.item.description}</p>
      </Show>
      <div class="shop-item__footer">
        <span class="shop-item__price">
          {props.item.price} {t('cashShop.currency')}
        </span>
        <Button variant="primary" disabled={props.isPending} onClick={handlePurchase}>
          {props.isPending ? <Spinner label={t('common.loading')} /> : t('cashShop.buy')}
        </Button>
      </div>
    </Card>
  );
}
