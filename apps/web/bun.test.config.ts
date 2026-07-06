// Configuration for bun test
// Exclude vitest component tests from bun test

export default {
  // Explicitly exclude vitest component tests that use @solidjs/testing-library
  exclude: [
    'node_modules',
    'dist',
    'src/components/ChecksumVerifier.test.tsx',
    'src/components/DownloadVerifier.test.tsx',
    'src/components/FeedItem.test.tsx',
    'src/components/ShopItem.test.tsx',
    'src/routes/CashShop.test.tsx',
  ],
};
