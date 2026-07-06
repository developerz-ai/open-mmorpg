/**
 * Download metadata — client installer links, checksums, and versions sourced from
 * deploy-time env. Falls back to static defaults when env is unset (local dev).
 * → docs/specs/web-client/downloads
 */
import { config, type OperatorConfig } from '../config.ts';

export interface PlatformDownload {
  platform: string;
  version: string;
  url: string;
  checksum: string;
}

/**
 * Latest client version. Returns '0.1.0' if not configured.
 */
export function clientVersion(cfg: OperatorConfig = config): string {
  return cfg.downloads.version ?? '0.1.0';
}

/**
 * All available platform downloads with URLs and checksums. Filters to platforms
 * that have both a URL and checksum configured; omits incomplete entries.
 */
export function platformDownloads(cfg: OperatorConfig = config): PlatformDownload[] {
  const { urls, checksums } = cfg.downloads;
  const version = clientVersion(cfg);

  const downloads: PlatformDownload[] = [];

  // Windows x64
  if (urls.windowsX64 && checksums.windowsX64) {
    downloads.push({
      platform: 'Windows x64',
      version,
      url: urls.windowsX64,
      checksum: checksums.windowsX64,
    });
  }

  // Windows ARM64
  if (urls.windowsArm64 && checksums.windowsArm64) {
    downloads.push({
      platform: 'Windows ARM64',
      version,
      url: urls.windowsArm64,
      checksum: checksums.windowsArm64,
    });
  }

  // macOS x64
  if (urls.macosX64 && checksums.macosX64) {
    downloads.push({
      platform: 'macOS x64',
      version,
      url: urls.macosX64,
      checksum: checksums.macosX64,
    });
  }

  // macOS ARM64
  if (urls.macosArm64 && checksums.macosArm64) {
    downloads.push({
      platform: 'macOS ARM64',
      version,
      url: urls.macosArm64,
      checksum: checksums.macosArm64,
    });
  }

  // Linux x64
  if (urls.linuxX64 && checksums.linuxX64) {
    downloads.push({
      platform: 'Linux x64',
      version,
      url: urls.linuxX64,
      checksum: checksums.linuxX64,
    });
  }

  return downloads;
}

/**
 * Fallback downloads for local dev when env is unset. Matches the static data
 * from the original Downloads page implementation.
 */
export function fallbackDownloads(): PlatformDownload[] {
  const version = '0.1.0';
  return [
    {
      platform: 'Windows x64',
      version,
      url: '#',
      checksum: 'a1b2c3d4e5f6',
    },
    {
      platform: 'Windows ARM64',
      version,
      url: '#',
      checksum: 'f6e5d4c3b2a1',
    },
    {
      platform: 'macOS x64',
      version,
      url: '#',
      checksum: '9a8b7c6d5e4f',
    },
    {
      platform: 'macOS ARM64',
      version,
      url: '#',
      checksum: '1a2b3c4d5e6f',
    },
    {
      platform: 'Linux x64',
      version,
      url: '#',
      checksum: '2b3c4d5e6f7a',
    },
  ];
}

/**
 * Get all platform downloads, falling back to static defaults when none are
 * configured via env.
 */
export function getAllDownloads(cfg: OperatorConfig = config): PlatformDownload[] {
  const configured = platformDownloads(cfg);
  if (configured.length > 0) return configured;
  return fallbackDownloads();
}
