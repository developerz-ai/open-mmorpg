import { describe, expect, test } from 'bun:test';
import { de } from './locales/de.ts';
import { en } from './locales/en.ts';
import { es } from './locales/es.ts';
import { fr } from './locales/fr.ts';
import { ja } from './locales/ja.ts';
import { zh } from './locales/zh.ts';
import { LOCALES, resolveLocale } from './locales.ts';
import { createTranslator } from './translator.ts';

describe('locales', () => {
  test('includes all supported locales', () => {
    expect(LOCALES).toContain('en');
    expect(LOCALES).toContain('de');
    expect(LOCALES).toContain('es');
    expect(LOCALES).toContain('fr');
    expect(LOCALES).toContain('ja');
    expect(LOCALES).toContain('zh');
  });

  test('resolves all supported locales', () => {
    expect(resolveLocale('en')).toBe('en');
    expect(resolveLocale('de')).toBe('de');
    expect(resolveLocale('es')).toBe('es');
    expect(resolveLocale('fr')).toBe('fr');
    expect(resolveLocale('ja')).toBe('ja');
    expect(resolveLocale('zh')).toBe('zh');
  });

  test('English catalog has all expected keys', () => {
    const t = createTranslator(en);
    expect(t('common.loading')).toBe('Loading…');
    expect(t('nav.home')).toBe('Home');
    expect(t('auth.login.heading')).toBe('Log in');
    expect(t('armory.heading')).toBe('Armory');
  });

  test('German catalog returns German strings', () => {
    const t = createTranslator(de);
    expect(t('common.loading')).toBe('Laden…');
    expect(t('nav.home')).toBe('Startseite');
    expect(t('auth.login.heading')).toBe('Anmelden');
    expect(t('armory.heading')).toBe('Rüstkammer');
  });

  test('German catalog interpolates params correctly', () => {
    const t = createTranslator(de);
    expect(t('common.gold', { amount: 42 })).toBe('42 Gold');
    expect(t('realm.title', { name: 'Azeroth' })).toBe('Azeroth');
    expect(t('auth.login.success', { name: 'Aria' })).toBe('Willkommen zurück, Aria.');
  });

  test('German catalog has same key structure as English', () => {
    const tEn = createTranslator(en);
    const tDe = createTranslator(de);

    // Sample a few keys across different sections
    const sampleKeys = [
      'common.loading',
      'nav.home',
      'auth.login.heading',
      'armory.character.level',
      'auction.heading',
      'feed.heading',
    ];

    for (const key of sampleKeys) {
      const enResult = tEn(key);
      const deResult = tDe(key);
      // Both should return strings, not missing-key markers
      expect(enResult).not.toMatch(/^⟦.*⟧$/);
      expect(deResult).not.toMatch(/^⟦.*⟧$/);
    }
  });

  test('missing keys render loudly in German', () => {
    const t = createTranslator(de);
    expect(t('does.not.exist')).toBe('⟦does.not.exist⟧');
  });

  test('Spanish catalog returns Spanish strings', () => {
    const t = createTranslator(es);
    expect(t('common.loading')).toBe('Cargando…');
    expect(t('nav.home')).toBe('Inicio');
    expect(t('auth.login.heading')).toBe('Iniciar sesión');
    expect(t('armory.heading')).toBe('Arsenal');
  });

  test('Spanish catalog interpolates params correctly', () => {
    const t = createTranslator(es);
    expect(t('common.gold', { amount: 42 })).toBe('42 oro');
    expect(t('realm.title', { name: 'Azeroth' })).toBe('Azeroth');
    expect(t('auth.login.success', { name: 'Aria' })).toBe('Bienvenido de vuelta, Aria.');
  });

  test('Spanish catalog has same key structure as English', () => {
    const tEn = createTranslator(en);
    const tEs = createTranslator(es);

    const sampleKeys = [
      'common.loading',
      'nav.home',
      'auth.login.heading',
      'armory.character.level',
      'auction.heading',
      'feed.heading',
    ];

    for (const key of sampleKeys) {
      const enResult = tEn(key);
      const esResult = tEs(key);
      expect(enResult).not.toMatch(/^⟦.*⟧$/);
      expect(esResult).not.toMatch(/^⟦.*⟧$/);
    }
  });

  test('missing keys render loudly in Spanish', () => {
    const t = createTranslator(es);
    expect(t('does.not.exist')).toBe('⟦does.not.exist⟧');
  });

  test('French catalog returns French strings', () => {
    const t = createTranslator(fr);
    expect(t('common.loading')).toBe('Chargement…');
    expect(t('nav.home')).toBe('Accueil');
    expect(t('auth.login.heading')).toBe('Connexion');
    expect(t('armory.heading')).toBe('Armurerie');
  });

  test('French catalog interpolates params correctly', () => {
    const t = createTranslator(fr);
    expect(t('common.gold', { amount: 42 })).toBe('42 Or');
    expect(t('realm.title', { name: 'Azeroth' })).toBe('Azeroth');
    expect(t('auth.login.success', { name: 'Aria' })).toBe('Bienvenue, Aria.');
  });

  test('French catalog has same key structure as English', () => {
    const tEn = createTranslator(en);
    const tFr = createTranslator(fr);

    const sampleKeys = [
      'common.loading',
      'nav.home',
      'auth.login.heading',
      'armory.character.level',
      'auction.heading',
      'feed.heading',
    ];

    for (const key of sampleKeys) {
      const enResult = tEn(key);
      const frResult = tFr(key);
      expect(enResult).not.toMatch(/^⟦.*⟧$/);
      expect(frResult).not.toMatch(/^⟦.*⟧$/);
    }
  });

  test('missing keys render loudly in French', () => {
    const t = createTranslator(fr);
    expect(t('does.not.exist')).toBe('⟦does.not.exist⟧');
  });

  test('Japanese catalog returns Japanese strings', () => {
    const t = createTranslator(ja);
    expect(t('common.loading')).toBe('読み込み中…');
    expect(t('nav.home')).toBe('ホーム');
    expect(t('auth.login.heading')).toBe('ログイン');
    expect(t('armory.heading')).toBe('武器庫');
  });

  test('Japanese catalog interpolates params correctly', () => {
    const t = createTranslator(ja);
    expect(t('common.gold', { amount: 42 })).toBe('42ゴールド');
    expect(t('realm.title', { name: 'Azeroth' })).toBe('Azeroth');
    expect(t('auth.login.success', { name: 'Aria' })).toBe('おかえりなさい、Aria。');
  });

  test('Japanese catalog has same key structure as English', () => {
    const tEn = createTranslator(en);
    const tJa = createTranslator(ja);

    const sampleKeys = [
      'common.loading',
      'nav.home',
      'auth.login.heading',
      'armory.character.level',
      'auction.heading',
      'feed.heading',
    ];

    for (const key of sampleKeys) {
      const enResult = tEn(key);
      const jaResult = tJa(key);
      expect(enResult).not.toMatch(/^⟦.*⟧$/);
      expect(jaResult).not.toMatch(/^⟦.*⟧$/);
    }
  });

  test('missing keys render loudly in Japanese', () => {
    const t = createTranslator(ja);
    expect(t('does.not.exist')).toBe('⟦does.not.exist⟧');
  });

  test('Chinese catalog returns Chinese strings', () => {
    const t = createTranslator(zh);
    expect(t('common.loading')).toBe('加载中…');
    expect(t('nav.home')).toBe('首页');
    expect(t('auth.login.heading')).toBe('登录');
    expect(t('armory.heading')).toBe('英雄榜');
  });

  test('Chinese catalog interpolates params correctly', () => {
    const t = createTranslator(zh);
    expect(t('common.gold', { amount: 42 })).toBe('42 金币');
    expect(t('realm.title', { name: 'Azeroth' })).toBe('Azeroth');
    expect(t('auth.login.success', { name: 'Aria' })).toBe('欢迎回来，Aria。');
  });

  test('Chinese catalog has same key structure as English', () => {
    const tEn = createTranslator(en);
    const tZh = createTranslator(zh);

    const sampleKeys = [
      'common.loading',
      'nav.home',
      'auth.login.heading',
      'armory.character.level',
      'auction.heading',
      'feed.heading',
    ];

    for (const key of sampleKeys) {
      const enResult = tEn(key);
      const zhResult = tZh(key);
      expect(enResult).not.toMatch(/^⟦.*⟧$/);
      expect(zhResult).not.toMatch(/^⟦.*⟧$/);
    }
  });

  test('missing keys render loudly in Chinese', () => {
    const t = createTranslator(zh);
    expect(t('does.not.exist')).toBe('⟦does.not.exist⟧');
  });
});
