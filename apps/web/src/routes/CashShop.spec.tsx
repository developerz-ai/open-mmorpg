import { render, screen, waitFor } from '@solidjs/testing-library';
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import type { JSX } from 'solid-js';
import { beforeEach, describe, expect, test, vi } from 'vitest';
import type { BuyIntent, BuyResult, ShopItem } from '../lib/cashShop';
import * as cashShopQueries from '../queries/useCashShop';
import CashShop from './CashShop';

let queryClient: QueryClient;

function wrapper(props: { children: JSX.Element }): JSX.Element {
  return <QueryClientProvider client={queryClient}>{props.children}</QueryClientProvider>;
}

/** Create a typed UseQueryResult mock for shop items. */
function mockShopItemsResult(
  overrides: Partial<{
    data: ShopItem[];
    isPending: boolean;
    isError: boolean;
    error: Error | null;
  }> = {},
): { data: ShopItem[] | undefined; isPending: boolean; isError: boolean; error: Error | null } {
  return {
    data: undefined,
    isPending: false,
    isError: false,
    error: null,
    ...overrides,
  };
}

/** Create a typed UseQueryResult mock for categories. */
function mockCategoriesResult(
  overrides: Partial<{
    data: string[];
    isPending: boolean;
    isError: boolean;
    error: Error | null;
  }> = {},
): { data: string[] | undefined; isPending: boolean; isError: boolean; error: Error | null } {
  return {
    data: undefined,
    isPending: false,
    isError: false,
    error: null,
    ...overrides,
  };
}

/** Create a typed UseMutationResult mock for buy shop item. */
function mockBuyMutationResult(
  overrides: Partial<{
    mutate: (intent: BuyIntent) => void;
    isPending: boolean;
    isSuccess: boolean;
    isError: boolean;
    data: BuyResult | undefined;
    error: Error | null;
  }> = {},
): {
  mutate: (intent: BuyIntent) => void;
  isPending: boolean;
  isSuccess: boolean;
  isError: boolean;
  data: BuyResult | undefined;
  error: Error | null;
} {
  return {
    mutate: vi.fn(),
    isPending: false,
    isSuccess: false,
    isError: false,
    data: undefined,
    error: null,
    ...overrides,
  };
}

/** Setup all three hook mocks with optional overrides. */
function setupMocks(
  itemsOverrides: Parameters<typeof mockShopItemsResult>[0] = {},
  categoriesOverrides: Parameters<typeof mockCategoriesResult>[0] = {},
  buyOverrides: Parameters<typeof mockBuyMutationResult>[0] = {},
) {
  vi.spyOn(cashShopQueries, 'useShopItems').mockReturnValue(mockShopItemsResult(itemsOverrides));
  vi.spyOn(cashShopQueries, 'useShopCategories').mockReturnValue(
    mockCategoriesResult(categoriesOverrides),
  );
  vi.spyOn(cashShopQueries, 'useBuyShopItem').mockReturnValue(mockBuyMutationResult(buyOverrides));
}

const mockShopItems: ShopItem[] = [
  { id: '1', name: 'XP Boost', category: 'boosts', price: 100, description: 'Double XP' },
  { id: '2', name: 'Cosmetic Hat', category: 'cosmetics', price: 250, description: null },
];

const mockCategories = ['boosts', 'cosmetics', 'mounts'];

describe('CashShop route component tests', () => {
  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    });
  });

  describe('loading state', () => {
    test('shows spinner when items are loading', () => {
      setupMocks(
        { isPending: true }, // items loading
        { data: mockCategories, isPending: false }, // categories loaded
      );

      render(() => <CashShop />, { wrapper });

      // Should show loading spinner
      expect(screen.getByText('Loading…')).toBeInTheDocument();
    });

    test('shows spinner when categories are loading', () => {
      setupMocks(
        { data: mockShopItems, isPending: false }, // items loaded
        { isPending: true }, // categories loading
      );

      render(() => <CashShop />, { wrapper });

      // Should show loading spinner for categories
      expect(screen.getByText('Loading…')).toBeInTheDocument();
    });
  });

  describe('error state', () => {
    test('shows error when items fail to load', async () => {
      const mockError = new Error('Network error');

      setupMocks(
        { isError: true, error: mockError }, // items error
        { data: mockCategories, isPending: false }, // categories loaded
      );

      render(() => <CashShop />, { wrapper });

      await waitFor(() => {
        expect(screen.getByText('Something went wrong. Please try again.')).toBeInTheDocument();
      });
    });

    test('shows items without categories toolbar when categories fail to load', async () => {
      const mockError = new Error('Network error');

      setupMocks(
        { data: mockShopItems, isPending: false }, // items loaded
        { isError: true, error: mockError }, // categories error
      );

      render(() => <CashShop />, { wrapper });

      await waitFor(() => {
        // Items should still render
        expect(screen.getByText('XP Boost')).toBeInTheDocument();
        // Categories toolbar should not be shown
        expect(screen.queryByText('All')).not.toBeInTheDocument();
      });
    });
  });

  describe('data state', () => {
    test('renders cash shop with items and categories', async () => {
      setupMocks(
        { data: mockShopItems, isPending: false }, // items loaded
        { data: mockCategories, isPending: false }, // categories loaded
      );

      render(() => <CashShop />, { wrapper });

      await waitFor(() => {
        // Check heading
        expect(screen.getByText('Cash Shop')).toBeInTheDocument();

        // Check category chips (using getAllByText since categories appear in both buttons and table)
        expect(screen.getAllByText('All')).toHaveLength(1);
        expect(screen.getAllByText('Boosts')).toHaveLength(2); // Once in button, once in table
        expect(screen.getAllByText('Cosmetics')).toHaveLength(2); // Once in button, once in table
        expect(screen.getAllByText('Mounts')).toHaveLength(1); // Only in button

        // Check item names
        expect(screen.getByText('XP Boost')).toBeInTheDocument();
        expect(screen.getByText('Cosmetic Hat')).toBeInTheDocument();
      });
    });

    test('shows empty state when no items', async () => {
      setupMocks(
        { data: [], isPending: false }, // empty items
        { data: mockCategories, isPending: false }, // categories loaded
      );

      render(() => <CashShop />, { wrapper });

      await waitFor(() => {
        expect(screen.getByText('No items in this category.')).toBeInTheDocument();
      });
    });

    test('shows success message after purchase', async () => {
      setupMocks(
        { data: mockShopItems, isPending: false }, // items loaded
        { data: mockCategories, isPending: false }, // categories loaded
        { isSuccess: true, data: { item: 'XP Boost', price: 100 } }, // successful purchase
      );

      render(() => <CashShop />, { wrapper });

      await waitFor(() => {
        expect(screen.getByText(/Purchased XP Boost for 100 credits/)).toBeInTheDocument();
      });
    });

    test('shows error message after failed purchase', async () => {
      const mockError = new Error('Insufficient funds');

      setupMocks(
        { data: mockShopItems, isPending: false }, // items loaded
        { data: mockCategories, isPending: false }, // categories loaded
        { isError: true, error: mockError }, // failed purchase
      );

      render(() => <CashShop />, { wrapper });

      await waitFor(() => {
        expect(screen.getByText('Something went wrong. Please try again.')).toBeInTheDocument();
      });
    });
  });
});
