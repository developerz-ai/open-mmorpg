// Configuration for bun test
// Exclude vitest component tests from bun test

export default {
  // Explicitly exclude vitest component tests
  exclude: [
    'node_modules',
    'dist',
    'src/components/ShopItem.test.tsx',
    'src/components/FeedItem.test.tsx',
    'src/routes/CashShop.test.tsx',
  ],
};
