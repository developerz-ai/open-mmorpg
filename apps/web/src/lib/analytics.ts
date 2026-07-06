/**
 * Operator analytics interface.
 *
 * Supports multiple analytics providers (Plausible, Google Analytics, custom).
 * Operators configure their provider via environment variables; this module
 * provides a unified interface for event tracking.
 *
 * @example
 * ```ts
 * import { trackEvent, trackPageView } from './lib/analytics';
 *
 * trackEvent('auction_purchase', { item_id: '123', price: 1000 });
 * trackPageView('/auction');
 * ```
 */

export interface AnalyticsProvider {
  /** Track a custom event. */
  trackEvent: (name: string, props?: Record<string, unknown>) => void;
  /** Track a page view. */
  trackPageView: (path: string, title?: string) => void;
  /** Initialize the provider (called once on app boot). */
  init?: () => void;
}

export interface AnalyticsConfig {
  /** Provider name: 'plausible', 'ga', 'custom', or 'none'. */
  provider: 'plausible' | 'ga' | 'custom' | 'none';
  /** Custom domain for self-hosted Plausible. */
  plausibleDomain?: string;
  /** Google Analytics measurement ID. */
  gaId?: string;
  /** Custom tracking endpoint (for 'custom' provider). */
  customEndpoint?: string;
}

/**
 * Get analytics config from environment variables.
 */
function getConfig(): AnalyticsConfig {
  const provider = (import.meta.env.VITE_ANALYTICS_PROVIDER ||
    'none') as AnalyticsConfig['provider'];
  return {
    provider,
    plausibleDomain: import.meta.env.VITE_PLAUSIBLE_DOMAIN,
    gaId: import.meta.env.VITE_GA_ID,
    customEndpoint: import.meta.env.VITE_ANALYTICS_ENDPOINT,
  };
}

/**
 * Create analytics provider based on config.
 */
function createProvider(config: AnalyticsConfig): AnalyticsProvider | null {
  switch (config.provider) {
    case 'plausible':
      return {
        trackEvent: (name, props) => {
          (
            window as typeof window & {
              plausible?: (e: string, p?: Record<string, unknown>) => void;
            }
          ).plausible?.(name, { props });
        },
        trackPageView: (path) => {
          (
            window as typeof window & {
              plausible?: (e: string, p?: Record<string, unknown>) => void;
            }
          ).plausible?.('pageview', { props: { path } });
        },
        init: () => {
          const script = document.createElement('script');
          script.async = true;
          script.defer = true;
          script.dataset.websiteId = import.meta.env.VITE_PLAUSIBLE_ID || '';
          script.src = config.plausibleDomain || 'https://plausible.io/js/script.js';
          document.head.appendChild(script);
        },
      };

    case 'ga':
      return {
        trackEvent: (name, props) => {
          (
            window as typeof window & {
              gtag?: (c: string, a: string, p: Record<string, unknown>) => void;
            }
          ).gtag?.('event', name, props || {});
        },
        trackPageView: (path, title) => {
          (
            window as typeof window & {
              gtag?: (c: 'config', a: string, p: Record<string, unknown>) => void;
            }
          ).gtag?.('config', config.gaId || '', { page_path: path, page_title: title });
        },
        init: () => {
          const script = document.createElement('script');
          script.async = true;
          script.src = `https://www.googletagmanager.com/gtag/js?id=${config.gaId}`;
          document.head.appendChild(script);

          (window as typeof window & { dataLayer: unknown[] }).dataLayer =
            (window as typeof window & { dataLayer: unknown[] }).dataLayer || [];
          (window as typeof window & { gtag: (...args: unknown[]) => void }).gtag = (...args) => {
            (window as typeof window & { dataLayer: unknown[] }).dataLayer.push(args);
          };
          (window as typeof window & { gtag: (...args: unknown[]) => void }).gtag('js', new Date());
          (window as typeof window & { gtag: (c: 'config', a: string) => void }).gtag(
            'config',
            config.gaId || '',
          );
        },
      };

    case 'custom':
      return {
        trackEvent: (name, props) => {
          if (config.customEndpoint) {
            fetch(config.customEndpoint, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ event: name, props }),
              keepalive: true,
            }).catch(() => {
              // Silently fail to not block UI
            });
          }
        },
        trackPageView: (path, title) => {
          if (config.customEndpoint) {
            fetch(config.customEndpoint, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ page: path, title }),
              keepalive: true,
            }).catch(() => {
              // Silently fail
            });
          }
        },
      };

    default:
      return null;
  }
}

let provider: AnalyticsProvider | null = null;

/**
 * Initialize analytics (call once on app boot).
 */
export function initAnalytics(): void {
  const config = getConfig();
  provider = createProvider(config);
  provider?.init?.();
}

/**
 * Track a custom event.
 * @example
 * trackEvent('auction_purchase', { item: 'Sword', price: 1000 });
 */
export function trackEvent(name: string, props?: Record<string, unknown>): void {
  provider?.trackEvent(name, props);
}

/**
 * Track a page view.
 * @example
 * trackPageView('/auction', 'Auction House');
 */
export function trackPageView(path: string, title?: string): void {
  provider?.trackPageView(path, title);
}

/**
 * Hook for React components to track events.
 * @example
 * function BuyButton({ item }) {
 *   const analytics = useAnalytics();
 *   return (
 *     <button onClick={() => analytics.trackEvent('buy_click', { item })}>
 *       Buy
 *     </button>
 *   );
 * }
 */
export function createAnalyticsHooks() {
  return {
    trackEvent,
    trackPageView,
  };
}

/**
 * React hook for analytics.
 * Re-export for convenience in components.
 */
export const useAnalytics = createAnalyticsHooks;
