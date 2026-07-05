import { createFormatters, createTranslator } from '@omm/i18n';
import { config } from '../config.ts';
import { en } from './catalog.ts';

/**
 * The app's translator and `Intl` formatters, bound to the operator's locale.
 * Copy comes from the catalog via `t()`; dates/numbers/money come from `fmt`
 * (never a hand-written format string in the catalog).
 * → docs/specs/web-client/i18n
 */
export const t = createTranslator(en);

/** Locale-aware `Intl` formatters (integer, gold, date, relative time, …). */
export const fmt = createFormatters(config.locale);
