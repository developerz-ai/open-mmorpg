import { describe, expect, test, beforeEach } from 'bun:test';
import { parseConfig } from '../config.ts';
import {
  clientVersion,
  platformDownloads,
  fallbackDownloads,
  getAllDownloads,
} from './downloads.ts';

describe('downloads', () => {
  describe('clientVersion', () => {
    test('returns configured version', () => {
      const raw = {
        VITE_DOWNLOAD_VERSION: '1.2.3',
      };
      const cfg = parseConfig(raw);
      expect(clientVersion(cfg)).toBe('1.2.3');
    });

    test('defaults to 0.1.0 when unset', () => {
      const raw = {};
      const cfg = parseConfig(raw);
      expect(clientVersion(cfg)).toBe('0.1.0');
    });
  });

  describe('platformDownloads', () => {
    test('returns empty array when no URLs configured', () => {
      const raw = {};
      const cfg = parseConfig(raw);
      expect(platformDownloads(cfg)).toEqual([]);
    });

    test('includes platforms with both URL and checksum', () => {
      const raw = {
        VITE_DOWNLOAD_URL_WINDOWS_X64: 'https://example.com/win.exe',
        VITE_DOWNLOAD_CHECKSUM_WINDOWS_X64: 'abc123',
        VITE_DOWNLOAD_URL_MACOS_ARM64: 'https://example.com/mac.dmg',
        VITE_DOWNLOAD_CHECKSUM_MACOS_ARM64: 'def456',
      };
      const cfg = parseConfig(raw);
      const downloads = platformDownloads(cfg);
      expect(downloads).toHaveLength(2);
      const win = downloads[0];
      if (win) {
        expect(win.platform).toBe('Windows x64');
        expect(win.url).toBe('https://example.com/win.exe');
        expect(win.checksum).toBe('abc123');
      }
      const mac = downloads[1];
      if (mac) {
        expect(mac.platform).toBe('macOS ARM64');
        expect(mac.url).toBe('https://example.com/mac.dmg');
        expect(mac.checksum).toBe('def456');
      }
    });

    test('omits platforms with URL but no checksum', () => {
      const raw = {
        VITE_DOWNLOAD_URL_WINDOWS_X64: 'https://example.com/win.exe',
        // no checksum
      };
      const cfg = parseConfig(raw);
      expect(platformDownloads(cfg)).toEqual([]);
    });

    test('omits platforms with checksum but no URL', () => {
      const raw = {
        // no URL
        VITE_DOWNLOAD_CHECKSUM_WINDOWS_X64: 'abc123',
      };
      const cfg = parseConfig(raw);
      expect(platformDownloads(cfg)).toEqual([]);
    });

    test('uses configured version for all entries', () => {
      const raw = {
        VITE_DOWNLOAD_VERSION: '2.0.0',
        VITE_DOWNLOAD_URL_LINUX_X64: 'https://example.com/linux.tar.gz',
        VITE_DOWNLOAD_CHECKSUM_LINUX_X64: 'ghi789',
      };
      const cfg = parseConfig(raw);
      const downloads = platformDownloads(cfg);
      expect(downloads).toHaveLength(1);
      const first = downloads[0];
      if (first) {
        expect(first.version).toBe('2.0.0');
      }
    });
  });

  describe('fallbackDownloads', () => {
    test('returns static defaults for all platforms', () => {
      const downloads = fallbackDownloads();
      expect(downloads).toHaveLength(5);

      const platforms = downloads.map((d) => d.platform);
      expect(platforms).toContain('Windows x64');
      expect(platforms).toContain('Windows ARM64');
      expect(platforms).toContain('macOS x64');
      expect(platforms).toContain('macOS ARM64');
      expect(platforms).toContain('Linux x64');
    });

    test('all fallback entries have version 0.1.0', () => {
      const downloads = fallbackDownloads();
      for (const dl of downloads) {
        expect(dl.version).toBe('0.1.0');
      }
    });

    test('all fallback entries have placeholder URLs', () => {
      const downloads = fallbackDownloads();
      for (const dl of downloads) {
        expect(dl.url).toBe('#');
      }
    });
  });

  describe('getAllDownloads', () => {
    test('returns configured downloads when present', () => {
      const raw = {
        VITE_DOWNLOAD_URL_WINDOWS_X64: 'https://example.com/win.exe',
        VITE_DOWNLOAD_CHECKSUM_WINDOWS_X64: 'abc123',
      };
      const cfg = parseConfig(raw);
      const downloads = getAllDownloads(cfg);
      expect(downloads).toHaveLength(1);
      const first = downloads[0];
      if (first) {
        expect(first.url).toBe('https://example.com/win.exe');
      }
    });

    test('returns fallback when no downloads configured', () => {
      const raw = {};
      const cfg = parseConfig(raw);
      const downloads = getAllDownloads(cfg);
      expect(downloads).toHaveLength(5);
      expect(downloads).toEqual(fallbackDownloads());
    });
  });
});
