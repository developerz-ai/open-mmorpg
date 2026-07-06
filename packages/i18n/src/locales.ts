/** Locales shipped in the box. Operators extend this by dropping a catalog. */
export const LOCALES = ['en', 'de', 'es', 'fr', 'ja', 'zh'] as const;

/** A supported locale tag. */
export type Locale = (typeof LOCALES)[number];

/** The fallback locale used when a requested one is unavailable. */
export const DEFAULT_LOCALE: Locale = 'en';

/** Narrow an arbitrary string to a known [`Locale`], or fall back to default. */
export function resolveLocale(tag: string): Locale {
  return (LOCALES as readonly string[]).includes(tag) ? (tag as Locale) : DEFAULT_LOCALE;
}
