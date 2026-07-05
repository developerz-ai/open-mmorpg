/* @refresh reload */
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import { render } from 'solid-js/web';
import { App } from './App.tsx';
import { config } from './config.ts';
import { applyBrand } from './lib/brand.ts';
import './styles.css';

applyBrand(config);

const root = document.getElementById('root');
if (!root) throw new Error('missing #root element');

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: 1, staleTime: 10_000 } },
});

render(
  () => (
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  ),
  root,
);
