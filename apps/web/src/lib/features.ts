import type { FeatureFlag } from '../config.ts';
import { config } from '../config.ts';

/**
 * Feature flags are UX; the server re-enforces. A hidden surface is also an off
 * endpoint — this only decides whether the web *offers* it.
 * → docs/specs/web-client/operator-brand
 */
export function isEnabled(flag: FeatureFlag): boolean {
  return config.features[flag];
}
