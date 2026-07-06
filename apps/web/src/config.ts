/**
 * Operator-brandable configuration — realm identity, brand palette (within the
 * dark tokens), API endpoints, and feature flags come from env at deploy, never
 * from code edits. Mirrors the data-driven core: one codebase, many branded
 * deployments. The env is **Zod-validated at boot** so a bad config fails loud
 * (a blank screen with a clear error) instead of a silent half-brand.
 * → docs/specs/web-client/operator-brand
 */
import { z } from 'zod';

/** `true`/`false` env strings → boolean, defaulting when unset. */
const flag = (fallback: boolean) =>
  z
    .enum(['true', 'false'])
    .optional()
    .transform((v) => (v === undefined ? fallback : v === 'true'));

/** An optional "R G B" channel triple (space-separated) for a token override. */
const rgbChannels = z
  .string()
  .regex(/^\d{1,3} \d{1,3} \d{1,3}$/, 'expected "R G B" channels')
  .optional();

const EnvSchema = z.object({
  VITE_REALM_NAME: z.string().min(1).default('Open-MMORPG'),
  VITE_REALM_TAGLINE: z.string().optional(),
  VITE_LOGO_URL: z.string().optional(),
  VITE_GATEWAY_URL: z.string().url().default('http://localhost:8080'),
  VITE_WORLDSVC_URL: z.string().url().default('http://localhost:8081'),
  VITE_LOCALE: z.enum(['en']).default('en'),
  VITE_BRAND_ACCENT: rgbChannels,
  VITE_BRAND_ACCENT_STRONG: rgbChannels,
  VITE_USE_MOCKS: flag(false),
  VITE_REGISTRATION_OPEN: flag(true),
  VITE_CASH_SHOP: flag(false),
  VITE_ARMORY_PUBLIC: flag(true),
  VITE_AUCTION_HOUSE: flag(true),
  VITE_WORLD_FEED: flag(true),
  // Download metadata
  VITE_DOWNLOAD_VERSION: z.string().optional(),
  VITE_DOWNLOAD_URL_WINDOWS_X64: z.string().url().optional(),
  VITE_DOWNLOAD_URL_WINDOWS_ARM64: z.string().url().optional(),
  VITE_DOWNLOAD_URL_MACOS_X64: z.string().url().optional(),
  VITE_DOWNLOAD_URL_MACOS_ARM64: z.string().url().optional(),
  VITE_DOWNLOAD_URL_LINUX_X64: z.string().url().optional(),
  VITE_DOWNLOAD_CHECKSUM_WINDOWS_X64: z.string().optional(),
  VITE_DOWNLOAD_CHECKSUM_WINDOWS_ARM64: z.string().optional(),
  VITE_DOWNLOAD_CHECKSUM_MACOS_X64: z.string().optional(),
  VITE_DOWNLOAD_CHECKSUM_MACOS_ARM64: z.string().optional(),
  VITE_DOWNLOAD_CHECKSUM_LINUX_X64: z.string().optional(),
});

export interface OperatorConfig {
  brand: {
    realmName: string;
    tagline?: string;
    logoUrl?: string;
    /** Accent token overrides ("R G B" channels), applied within the dark palette. */
    accent?: string;
    accentStrong?: string;
  };
  locale: 'en';
  endpoints: { gatewayUrl: string; worldsvcUrl: string };
  /** Route the API client to the in-memory mock backend (server not yet live). */
  useMocks: boolean;
  features: {
    registrationOpen: boolean;
    cashShop: boolean;
    armoryPublic: boolean;
    auctionHouse: boolean;
    worldFeed: boolean;
  };
  downloads: {
    version?: string;
    urls: {
      windowsX64?: string;
      windowsArm64?: string;
      macosX64?: string;
      macosArm64?: string;
      linuxX64?: string;
    };
    checksums: {
      windowsX64?: string;
      windowsArm64?: string;
      macosX64?: string;
      macosArm64?: string;
      linuxX64?: string;
    };
  };
}

/** A named feature flag — the keys of `OperatorConfig['features']`. */
export type FeatureFlag = keyof OperatorConfig['features'];

/** Parse a raw env bag into a validated config. Exported for tests. */
export function parseConfig(raw: Record<string, unknown>): OperatorConfig {
  const e = EnvSchema.parse(raw);
  return {
    brand: {
      realmName: e.VITE_REALM_NAME,
      tagline: e.VITE_REALM_TAGLINE,
      logoUrl: e.VITE_LOGO_URL,
      accent: e.VITE_BRAND_ACCENT,
      accentStrong: e.VITE_BRAND_ACCENT_STRONG,
    },
    locale: e.VITE_LOCALE,
    endpoints: { gatewayUrl: e.VITE_GATEWAY_URL, worldsvcUrl: e.VITE_WORLDSVC_URL },
    useMocks: e.VITE_USE_MOCKS,
    features: {
      registrationOpen: e.VITE_REGISTRATION_OPEN,
      cashShop: e.VITE_CASH_SHOP,
      armoryPublic: e.VITE_ARMORY_PUBLIC,
      auctionHouse: e.VITE_AUCTION_HOUSE,
      worldFeed: e.VITE_WORLD_FEED,
    },
    downloads: {
      version: e.VITE_DOWNLOAD_VERSION,
      urls: {
        windowsX64: e.VITE_DOWNLOAD_URL_WINDOWS_X64,
        windowsArm64: e.VITE_DOWNLOAD_URL_WINDOWS_ARM64,
        macosX64: e.VITE_DOWNLOAD_URL_MACOS_X64,
        macosArm64: e.VITE_DOWNLOAD_URL_MACOS_ARM64,
        linuxX64: e.VITE_DOWNLOAD_URL_LINUX_X64,
      },
      checksums: {
        windowsX64: e.VITE_DOWNLOAD_CHECKSUM_WINDOWS_X64,
        windowsArm64: e.VITE_DOWNLOAD_CHECKSUM_WINDOWS_ARM64,
        macosX64: e.VITE_DOWNLOAD_CHECKSUM_MACOS_X64,
        macosArm64: e.VITE_DOWNLOAD_CHECKSUM_MACOS_ARM64,
        linuxX64: e.VITE_DOWNLOAD_CHECKSUM_LINUX_X64,
      },
    },
  };
}

export const config: OperatorConfig = parseConfig(import.meta.env as Record<string, unknown>);
