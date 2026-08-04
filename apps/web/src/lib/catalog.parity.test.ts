import { describe, expect, test } from 'bun:test';
import { type Catalog, flatten, LOCALES, type Locale } from '@omm/i18n';
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

/**
 * A key missing from a catalog renders `⟦key⟧` — loud by design, and impossible to miss
 * in development. In a shipped build nobody is watching, so the marker simply becomes the
 * copy: the world feed's filter, pagination and share controls were authored in English
 * only, and all twelve other locales rendered `⟦feed.pagination.next⟧` to real visitors —
 * including into `aria-label`, where a screen reader read the marker aloud.
 *
 * Nothing compared the catalogs, so nothing caught it. This is that comparison.
 */
const CATALOGS: Record<Locale, Catalog> = { en, de, es, fr, ja, zh, ko, ru, pt, it, pl, tr, ar };

describe('catalog parity', () => {
  const reference = flatten(en);
  const referenceKeys = Object.keys(reference).sort();

  test('every locale the app offers has a catalog behind it', () => {
    for (const locale of LOCALES) expect(CATALOGS[locale]).toBeDefined();
  });

  for (const locale of LOCALES) {
    test(`${locale} carries every key English carries`, () => {
      const missing = referenceKeys.filter((key) => !(key in flatten(CATALOGS[locale])));
      expect(missing).toEqual([]);
    });

    test(`${locale} carries no key English does not`, () => {
      // An extra key is unreachable copy: no call site asks for it, because every call
      // site is written against English. It is dead weight that reads as coverage.
      const extra = Object.keys(flatten(CATALOGS[locale])).filter((key) => !(key in reference));
      expect(extra).toEqual([]);
    });

    test(`${locale} keeps every interpolation placeholder`, () => {
      // '{count} members' translated without its {count} silently drops the number.
      const catalog = flatten(CATALOGS[locale]);
      const dropped: string[] = [];
      for (const key of referenceKeys) {
        const slots = [...(reference[key]?.matchAll(/\{(\w+)\}/g) ?? [])].map((m) => m[1]).sort();
        if (slots.length === 0) continue;
        const mine = [...(catalog[key]?.matchAll(/\{(\w+)\}/g) ?? [])].map((m) => m[1]).sort();
        if (slots.join(',') !== mine.join(','))
          dropped.push(`${key}: expected {${slots}}, got {${mine}}`);
      }
      expect(dropped).toEqual([]);
    });
  }
});
