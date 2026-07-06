// Setup file for vitest component tests
import { vi } from 'vitest';
import '@testing-library/jest-dom';

// Mock the config module to prevent it from loading during tests
vi.mock('../config', () => ({
  config: {
    brand: {
      realmName: 'Test Realm',
      accent: '255 0 0',
      accentStrong: '255 100 100',
    },
    locale: 'en',
    endpoints: {
      gatewayUrl: 'http://localhost:8080',
      worldsvcUrl: 'http://localhost:8081',
    },
    useMocks: false,
    features: {
      registrationOpen: true,
      cashShop: false,
      armoryPublic: true,
      auctionHouse: true,
      worldFeed: true,
    },
    downloads: {
      version: '1.0.0',
      urls: {},
      checksums: {},
    },
  },
  parseConfig: (raw: Record<string, unknown>) => ({
    brand: {
      realmName: 'Test Realm',
      accent: '255 0 0',
      accentStrong: '255 100 100',
    },
    locale: 'en',
    endpoints: {
      gatewayUrl: 'http://localhost:8080',
      worldsvcUrl: 'http://localhost:8081',
    },
    useMocks: false,
    features: {
      registrationOpen: true,
      cashShop: false,
      armoryPublic: true,
      auctionHouse: true,
      worldFeed: true,
    },
    downloads: {
      version: '1.0.0',
      urls: {},
      checksums: {},
    },
  }),
}));

// Mock cashShop module to avoid zod validation issues in tests
vi.mock('../lib/cashShop', () => ({
  ShopItemType: {},
  fetchShopItems: async () => [],
  fetchShopCategories: async () => [],
  buyShopItem: async () => ({ item: 'Test Item', price: 100 }),
}));
