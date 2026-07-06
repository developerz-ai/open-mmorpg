import { defineConfig } from 'vitest/config';
import solid from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solid()],
  test: {
    environment: 'happy-dom',
    include: ['src/components/**/*.spec.tsx', 'src/routes/**/*.spec.tsx'],
    exclude: ['node_modules', 'dist', 'e2e'],
    globals: true,
    setupFiles: ['./src/test/vitest-setup.ts'],
    deps: {
      inline: ['solid-js', '@solidjs/router', '@tanstack/solid-query'],
    },
  },
  resolve: {
    conditions: ['browser', 'development'],
  },
});
