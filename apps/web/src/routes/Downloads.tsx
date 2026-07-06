import { Button, Card } from '@omm/ui';
import { A } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { For } from 'solid-js';
import { t } from '../lib/i18n.ts';

interface Platform {
  id: string;
  name: string;
  arch: string[];
}

interface Download {
  platform: string;
  version: string;
  size: string;
  checksum: string;
  url: string;
}

const _PLATFORMS: Platform[] = [
  { id: 'windows', name: 'Windows', arch: ['x64', 'arm64'] },
  { id: 'macos', name: 'macOS', arch: ['x64', 'arm64'] },
  { id: 'linux', name: 'Linux', arch: ['x64'] },
];

const DOWNLOADS: Download[] = [
  {
    platform: 'Windows x64',
    version: '0.1.0',
    size: '45 MB',
    checksum: 'a1b2c3d4e5f6',
    url: '#',
  },
  {
    platform: 'Windows ARM64',
    version: '0.1.0',
    size: '42 MB',
    checksum: 'f6e5d4c3b2a1',
    url: '#',
  },
  {
    platform: 'macOS x64',
    version: '0.1.0',
    size: '48 MB',
    checksum: '9a8b7c6d5e4f',
    url: '#',
  },
  {
    platform: 'macOS ARM64',
    version: '0.1.0',
    size: '44 MB',
    checksum: '1a2b3c4d5e6f',
    url: '#',
  },
  {
    platform: 'Linux x64',
    version: '0.1.0',
    size: '43 MB',
    checksum: '2b3c4d5e6f7a',
    url: '#',
  },
];

const REQUIREMENTS = [
  { os: 'Windows 10+', specs: 'Intel/AMD CPU, 4GB RAM, 500MB disk' },
  { os: 'macOS 11+', specs: 'Intel/Apple CPU, 4GB RAM, 500MB disk' },
  { os: 'Linux (Ubuntu 20.04+)', specs: 'x64 CPU, 4GB RAM, 500MB disk' },
];

/**
 * Downloads page — installer links, system requirements, checksums.
 * Thin: static data rendered through `t()` copy, no backend calls.
 */
export default function Downloads(): JSX.Element {
  return (
    <div class="stack">
      <Card title={t('downloads.heading')}>
        <p class="text-fg-muted">{t('downloads.body')}</p>
      </Card>

      <Card title={t('downloads.installers')} class="downloads">
        <p class="text-fg-muted downloads__note">{t('downloads.httpsNote')}</p>
        <div class="downloads__list">
          <For each={DOWNLOADS}>
            {(dl) => (
              <div class="downloads__item">
                <div class="downloads__info">
                  <span class="downloads__platform">{dl.platform}</span>
                  <span class="downloads__meta text-fg-muted">
                    {t('downloads.version')} {dl.version} · {dl.size}
                  </span>
                </div>
                <div class="downloads__actions">
                  <Button variant="primary" onClick={() => (window.location.href = dl.url)}>
                    {t('downloads.download')}
                  </Button>
                  <button
                    type="button"
                    class="downloads__checksum"
                    onClick={() => navigator.clipboard.writeText(dl.checksum)}
                  >
                    {t('downloads.checksum')} {dl.checksum}
                  </button>
                </div>
              </div>
            )}
          </For>
        </div>
      </Card>

      <Card title={t('downloads.requirements')} class="requirements">
        <table class="requirements__table">
          <thead>
            <tr>
              <th>{t('downloads.os')}</th>
              <th>{t('downloads.specs')}</th>
            </tr>
          </thead>
          <tbody>
            <For each={REQUIREMENTS}>
              {(req) => (
                <tr>
                  <td>{req.os}</td>
                  <td class="text-fg-muted">{req.specs}</td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </Card>

      <Card title={t('downloads.verification')} class="verification">
        <p class="text-fg-muted">{t('downloads.verificationBody')}</p>
        <pre class="verification__code">sha256sum {t('downloads.filenameExample')}</pre>
        <p class="text-fg-muted verification__hint">{t('downloads.verificationHint')}</p>
      </Card>

      <div class="actions">
        <A href="/">
          <Button variant="ghost">{t('downloads.backToHome')}</Button>
        </A>
      </div>
    </div>
  );
}
