import { createFormatters, createTranslator } from '@omm/i18n';
import { config } from '../config.ts';
import { ar } from './catalog.ar.ts';
import { de } from './catalog.de.ts';
import { es } from './catalog.es.ts';
import { fr } from './catalog.fr.ts';
import { it } from './catalog.it.ts';
import { ja } from './catalog.ja.ts';
import { ko } from './catalog.ko.ts';
import { pl } from './catalog.pl.ts';
import { pt } from './catalog.pt.ts';
import { ru } from './catalog.ru.ts';
import { tr } from './catalog.tr.ts';
import { en } from './catalog.ts';
import { zh } from './catalog.zh.ts';

/** Catalog map for all supported locales. */
const catalogs = {
  ar,
  de,
  en,
  es,
  fr,
  it,
  ja,
  ko,
  pl,
  pt,
  ru,
  tr,
  zh,
} as const;

/**
 * The app's translator and `Intl` formatters, bound to the operator's locale.
 * Copy comes from the catalog via `t()`; dates/numbers/money come from `fmt`
 * (never a hand-written format string in the catalog).
 * → docs/specs/web-client/i18n
 */
export const t = createTranslator(catalogs[config.locale]);

/** Locale-aware `Intl` formatters (integer, gold, date, relative time, …). */
export const fmt = createFormatters(config.locale);
