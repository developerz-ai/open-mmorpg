import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

describe('ShopItem component exports', () => {
  test('ShopItem.tsx exists and exports ShopItem', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('export function ShopItem');
    expect(content).toContain('ShopItemProps');
  });

  test('ShopItem exports ShopItemProps interface', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('export interface ShopItemProps');
    expect(content).toContain('item: ShopItemType');
    expect(content).toContain('isPending: boolean');
    expect(content).toContain('onPurchase: (itemId: string) => void');
  });
});

describe('ShopItem component structure', () => {
  test('ShopItem uses Card component', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('Card');
    expect(content).toContain('shop-item');
  });

  test('ShopItem uses Badge for category', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('Badge');
    expect(content).toContain('tone=');
  });

  test('ShopItem uses Button for purchase action', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('Button');
    expect(content).toContain('onClick={handlePurchase}');
  });

  test('ShopItem uses Spinner during pending state', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('Spinner');
  });

  test('ShopItem displays item name', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('shop-item__name');
    expect(content).toContain('props.item.name');
  });

  test('ShopItem displays item description conditionally', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('Show when={props.item.description}');
    expect(content).toContain('shop-item__description');
  });

  test('ShopItem displays price', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('shop-item__price');
    expect(content).toContain('props.item.price');
  });

  test('ShopItem uses i18n strings for UI', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('cashShop.category');
    expect(content).toContain("t('cashShop.buy')");
  });
});

describe('ShopItem purchase intent', () => {
  test('ShopItem disables button when purchase pending', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('disabled={props.isPending}');
  });

  test('ShopItem calls onPurchase with itemId on button click', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('props.onPurchase(props.item.id)');
    expect(content).toContain('handlePurchase');
  });

  test('ShopItem shows Spinner when pending', () => {
    const componentPath = join(import.meta.dir, 'ShopItem.tsx');
    const content = readFileSync(componentPath, 'utf-8');
    expect(content).toContain('{props.isPending ? <Spinner');
    expect(content).toContain("t('cashShop.buy')");
  });
});
