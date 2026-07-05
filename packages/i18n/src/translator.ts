import { type Catalog, flatten } from './catalog.ts';
import { interpolate } from './interpolate.ts';

/** Translate a dot-key to a message, interpolating optional params. */
export type Translate = (key: string, params?: Record<string, string | number>) => string;

/**
 * Build a translator from a nested catalog.
 *
 * A missing key renders **loudly** as `⟦key⟧` — never a silent blank — so gaps
 * are caught in review and screenshots instead of shipping as empty UI.
 */
export function createTranslator(catalog: Catalog): Translate {
  const flat = flatten(catalog);
  return (key, params) => {
    const template = flat[key];
    if (template === undefined) return `⟦${key}⟧`;
    return interpolate(template, params);
  };
}
