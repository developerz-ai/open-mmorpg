import { describe, expect, test, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import type { JSX } from 'solid-js';
import CashShop from './CashShop';
import * as cashShopQueries from '../queries/useCashShop';
import type { ShopItem } from '../lib/cashShop';

let queryClient: QueryClient;

function wrapper(props: { children: JSX.Element }): JSX.Element {
  return (
    <QueryClientProvider client={queryClient}>
      {props.children}
    </QueryClientProvider>
  );
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
      // Mock pending state for items
      vi.spyOn(cashShopQueries, 'useShopItems').mockReturnValue({
        data: undefined,
        isPending: true,
        isError: false,
        error: null,
      } as any);

      // Mock categories as loaded
      vi.spyOn(cashShopQueries, 'useShopCategories').mockReturnValue({
        data: mockCategories,
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock buy mutation
      vi.spyOn(cashShopQueries, 'useBuyShopItem').mockReturnValue({
        mutate: vi.fn(),
        isPending: false,
        isSuccess: false,
        isError: false,
        data: undefined,
        error: null,
      } as any);

      render(() => <CashShop />, { wrapper });

      // Should show loading spinner
      expect(screen.getByText('Loading…')).toBeInTheDocument();
    });

    test('shows spinner when categories are loading', () => {
      // Mock items as loaded
      vi.spyOn(cashShopQueries, 'useShopItems').mockReturnValue({
        data: mockShopItems,
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock pending state for categories
      vi.spyOn(cashShopQueries, 'useShopCategories').mockReturnValue({
        data: undefined,
        isPending: true,
        isError: false,
        error: null,
      } as any);

      // Mock buy mutation
      vi.spyOn(cashShopQueries, 'useBuyShopItem').mockReturnValue({
        mutate: vi.fn(),
        isPending: false,
        isSuccess: false,
        isError: false,
        data: undefined,
        error: null,
      } as any);

      render(() => <CashShop />, { wrapper });

      // Should show loading spinner for categories
      expect(screen.getByText('Loading…')).toBeInTheDocument();
    });
  });

  describe('error state', () => {
    test('shows error when items fail to load', async () => {
      const mockError = new Error('Network error');

      // Mock error state for items
      vi.spyOn(cashShopQueries, 'useShopItems').mockReturnValue({
        data: undefined,
        isPending: false,
        isError: true,
        error: mockError,
      } as any);

      // Mock categories as loaded
      vi.spyOn(cashShopQueries, 'useShopCategories').mockReturnValue({
        data: mockCategories,
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock buy mutation
      vi.spyOn(cashShopQueries, 'useBuyShopItem').mockReturnValue({
        mutate: vi.fn(),
        isPending: false,
        isSuccess: false,
        isError: false,
        data: undefined,
        error: null,
      } as any);

      render(() => <CashShop />, { wrapper });

      await waitFor(() => {
        expect(screen.getByText('Something went wrong. Please try again.')).toBeInTheDocument();
      });
    });

    test('shows items without categories toolbar when categories fail to load', async () => {
      const mockError = new Error('Network error');

      // Mock items as loaded
      vi.spyOn(cashShopQueries, 'useShopItems').mockReturnValue({
        data: mockShopItems,
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock error state for categories
      vi.spyOn(cashShopQueries, 'useShopCategories').mockReturnValue({
        data: undefined,
        isPending: false,
        isError: true,
        error: mockError,
      } as any);

      // Mock buy mutation
      vi.spyOn(cashShopQueries, 'useBuyShopItem').mockReturnValue({
        mutate: vi.fn(),
        isPending: false,
        isSuccess: false,
        isError: false,
        data: undefined,
        error: null,
      } as any);

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
      // Mock items as loaded
      vi.spyOn(cashShopQueries, 'useShopItems').mockReturnValue({
        data: mockShopItems,
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock categories as loaded
      vi.spyOn(cashShopQueries, 'useShopCategories').mockReturnValue({
        data: mockCategories,
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock buy mutation
      vi.spyOn(cashShopQueries, 'useBuyShopItem').mockReturnValue({
        mutate: vi.fn(),
        isPending: false,
        isSuccess: false,
        isError: false,
        data: undefined,
        error: null,
      } as any);

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
      // Mock empty items array
      vi.spyOn(cashShopQueries, 'useShopItems').mockReturnValue({
        data: [],
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock categories as loaded
      vi.spyOn(cashShopQueries, 'useShopCategories').mockReturnValue({
        data: mockCategories,
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock buy mutation
      vi.spyOn(cashShopQueries, 'useBuyShopItem').mockReturnValue({
        mutate: vi.fn(),
        isPending: false,
        isSuccess: false,
        isError: false,
        data: undefined,
        error: null,
      } as any);

      render(() => <CashShop />, { wrapper });

      await waitFor(() => {
        expect(screen.getByText('No items in this category.')).toBeInTheDocument();
      });
    });

    test('shows success message after purchase', async () => {
      // Mock items as loaded
      vi.spyOn(cashShopQueries, 'useShopItems').mockReturnValue({
        data: mockShopItems,
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock categories as loaded
      vi.spyOn(cashShopQueries, 'useShopCategories').mockReturnValue({
        data: mockCategories,
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock successful purchase
      vi.spyOn(cashShopQueries, 'useBuyShopItem').mockReturnValue({
        mutate: vi.fn(),
        isPending: false,
        isSuccess: true,
        isError: false,
        data: { item: 'XP Boost', price: 100 },
        error: null,
      } as any);

      render(() => <CashShop />, { wrapper });

      await waitFor(() => {
        expect(screen.getByText(/Purchased XP Boost for 100 credits/)).toBeInTheDocument();
      });
    });

    test('shows error message after failed purchase', async () => {
      const mockError = new Error('Insufficient funds');

      // Mock items as loaded
      vi.spyOn(cashShopQueries, 'useShopItems').mockReturnValue({
        data: mockShopItems,
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock categories as loaded
      vi.spyOn(cashShopQueries, 'useShopCategories').mockReturnValue({
        data: mockCategories,
        isPending: false,
        isError: false,
        error: null,
      } as any);

      // Mock failed purchase
      vi.spyOn(cashShopQueries, 'useBuyShopItem').mockReturnValue({
        mutate: vi.fn(),
        isPending: false,
        isSuccess: false,
        isError: true,
        data: undefined,
        error: mockError,
      } as any);

      render(() => <CashShop />, { wrapper });

      await waitFor(() => {
        expect(screen.getByText('Something went wrong. Please try again.')).toBeInTheDocument();
      });
    });
  });
});
