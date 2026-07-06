import { render, screen } from '@solidjs/testing-library';
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import type { JSX } from 'solid-js';
import { beforeEach, describe, expect, test } from 'vitest';
import type { ShopItem as ShopItemType } from '../lib/cashShop';
import { ShopItem } from './ShopItem';

let queryClient: QueryClient;

function wrapper(props: { children: JSX.Element }): JSX.Element {
  return <QueryClientProvider client={queryClient}>{props.children}</QueryClientProvider>;
}

describe('ShopItem component tests', () => {
  const mockItem: ShopItemType = {
    id: '1',
    name: 'Test XP Boost',
    category: 'boosts',
    price: 100,
    description: 'Double XP for 1 hour',
  };

  const mockItemWithoutDescription: ShopItemType = {
    id: '2',
    name: 'Test Mount',
    category: 'mounts',
    price: 500,
    description: null,
  };

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    });
  });

  test('renders item with all fields', () => {
    render(() => <ShopItem item={mockItem} isPending={false} onPurchase={() => {}} />, { wrapper });

    // Check item name
    expect(screen.getByText('Test XP Boost')).toBeInTheDocument();

    // Check category badge (the component uses t() to translate the category)
    expect(screen.getByText('Boosts')).toBeInTheDocument();

    // Check price
    expect(screen.getByText(/100/)).toBeInTheDocument();

    // Check buy button - should show "Buy" text
    expect(screen.getByText('Buy')).toBeInTheDocument();
  });

  test('renders item without description', () => {
    render(
      () => <ShopItem item={mockItemWithoutDescription} isPending={false} onPurchase={() => {}} />,
      { wrapper },
    );

    // Check item name
    expect(screen.getByText('Test Mount')).toBeInTheDocument();

    // Description should not be shown
    expect(screen.queryByText('Double XP for 1 hour')).not.toBeInTheDocument();
  });

  test('shows spinner when pending', () => {
    const { container } = render(
      () => <ShopItem item={mockItem} isPending={true} onPurchase={() => {}} />,
      { wrapper },
    );

    // Button should be disabled
    const button = container.querySelector('button[disabled]');
    expect(button).toBeInTheDocument();

    // Should show loading text
    expect(screen.getByText('Loading…')).toBeInTheDocument();
  });

  test('calls onPurchase when button clicked', () => {
    let clickedItemId: string | undefined;

    render(
      () => (
        <ShopItem
          item={mockItem}
          isPending={false}
          onPurchase={(itemId) => {
            clickedItemId = itemId;
          }}
        />
      ),
      { wrapper },
    );

    const buyButton = screen.getByText('Buy');
    buyButton.click();

    expect(clickedItemId).toBe('1');
  });

  test('does not call onPurchase when disabled', () => {
    let clicked = false;

    const { container } = render(
      () => (
        <ShopItem
          item={mockItem}
          isPending={true}
          onPurchase={() => {
            clicked = true;
          }}
        />
      ),
      { wrapper },
    );

    // Find disabled button
    const button = container.querySelector('button[disabled]');
    expect(button).toBeInTheDocument();

    // Clicking disabled button should not trigger purchase
    if (button) {
      button.click();
      expect(clicked).toBe(false);
    }
  });
});
