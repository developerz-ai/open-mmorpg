import { describe, expect, test } from 'bun:test';
import { parseConfig } from './config.ts';

describe('operator config (Zod at boot)', () => {
  test('applies safe defaults for an empty env', () => {
    const c = parseConfig({});
    expect(c.brand.realmName).toBe('Open-MMORPG');
    expect(c.endpoints.gatewayUrl).toBe('http://localhost:8080');
    expect(c.features.registrationOpen).toBe(true);
    expect(c.features.cashShop).toBe(false);
  });

  test('parses brand, endpoints, and feature flags from env', () => {
    const c = parseConfig({
      VITE_REALM_NAME: 'Ashenrealm',
      VITE_REALM_TAGLINE: 'Where legends fall.',
      VITE_GATEWAY_URL: 'https://gw.ashen.gg',
      VITE_WORLDSVC_URL: 'https://world.ashen.gg',
      VITE_REGISTRATION_OPEN: 'false',
      VITE_CASH_SHOP: 'true',
      VITE_BRAND_ACCENT: '220 40 40',
    });
    expect(c.brand.realmName).toBe('Ashenrealm');
    expect(c.brand.tagline).toBe('Where legends fall.');
    expect(c.brand.accent).toBe('220 40 40');
    expect(c.endpoints.worldsvcUrl).toBe('https://world.ashen.gg');
    expect(c.features.registrationOpen).toBe(false);
    expect(c.features.cashShop).toBe(true);
  });

  test('fails loud on a malformed gateway url', () => {
    expect(() => parseConfig({ VITE_GATEWAY_URL: 'not-a-url' })).toThrow();
  });

  test('fails loud on a malformed brand accent', () => {
    expect(() => parseConfig({ VITE_BRAND_ACCENT: '#ff0000' })).toThrow();
  });
});
