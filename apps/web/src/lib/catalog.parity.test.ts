import { describe, expect, test } from 'bun:test';
import { type Catalog, createTranslator, flatten, LOCALES, type Locale } from '@omm/i18n';
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

// Independent of English: deleting these controls from every catalog must still fail.
const REQUIRED_FEED_KEYS = [
  'feed.filter.label',
  'feed.filter.all',
  'feed.filter.bossKill',
  'feed.filter.worldBossSpawn',
  'feed.filter.factionShift',
  'feed.filter.bountyPosted',
  'feed.filter.milestone',
  'feed.pagination.page',
  'feed.pagination.prev',
  'feed.pagination.next',
  'feed.pagination.showing',
  'feed.autoRefresh.on',
  'feed.autoRefresh.off',
  'feed.newBadge',
  'feed.share',
  'feed.shareEvent',
] as const;

describe('catalog parity', () => {
  const reference = flatten(en);
  const referenceKeys = Object.keys(reference).sort();

  test('every locale the app offers has a catalog behind it', () => {
    for (const locale of LOCALES) expect(CATALOGS[locale]).toBeDefined();
  });

  for (const locale of LOCALES) {
    test(`${locale} renders every required feed control, including accessible share copy`, () => {
      const t = createTranslator(CATALOGS[locale]);
      for (const key of REQUIRED_FEED_KEYS) {
        const translated = t(key, { current: 1, total: 2, start: 1, end: 2 });
        expect(translated.trim().length).toBeGreaterThan(0);
        expect(translated).not.toMatch(/[⟦⟧]/);
      }
    });

    test(`${locale} renders missing keys loudly without English fallback`, () => {
      expect(createTranslator(CATALOGS[locale])('does.not.exist')).toBe('⟦does.not.exist⟧');
      const partial = { ...CATALOGS[locale], common: {} };
      expect(createTranslator(partial)('common.loading')).toBe('⟦common.loading⟧');
    });

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

// Current shipped copy, not duplicate package catalogs or stale translation expectations.
const COPY_SAMPLES = [
  ['en', 'Loading…', 'Home', 'Log in', 'Armory', '42 gold', 'Welcome back, Aria.'],
  [
    'de',
    'Laden…',
    'Startseite',
    'Anmelden',
    'Rüstungskammer',
    '42 Gold',
    'Willkommen zurück, Aria.',
  ],
  [
    'es',
    'Cargando…',
    'Inicio',
    'Iniciar sesión',
    'Arsenal',
    '42 oro',
    'Bienvenido de vuelta, Aria.',
  ],
  ['fr', 'Chargement…', 'Accueil', 'Se connecter', 'Armurerie', '42 or', 'Bon retour, Aria.'],
  ['ja', '読み込み中…', 'ホーム', 'ログイン', '武器庫', '42ゴールド', 'おかえりなさい、Aria。'],
  ['zh', '加载中…', '首页', '登录', '军械库', '42 金币', '欢迎回来，Aria。'],
] as const;

describe('shipped translations', () => {
  for (const [locale, loading, home, login, armory, gold, welcome] of COPY_SAMPLES) {
    test(`${locale} translates representative copy and interpolates params`, () => {
      const t = createTranslator(CATALOGS[locale]);
      expect(t('common.loading')).toBe(loading);
      expect(t('nav.home')).toBe(home);
      expect(t('auth.login.heading')).toBe(login);
      expect(t('armory.heading')).toBe(armory);
      expect(t('common.gold', { amount: 42 })).toBe(gold);
      expect(t('realm.title', { name: 'Aria' })).toBe('Aria');
      expect(t('auth.login.success', { name: 'Aria' })).toBe(welcome);
    });
  }
});
