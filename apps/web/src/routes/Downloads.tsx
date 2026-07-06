import { Button, Card } from '@omm/ui';
import { A } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { For } from 'solid-js';
import { DownloadVerifier } from '../components/DownloadVerifier.tsx';
import { getAllDownloads } from '../lib/downloads.ts';
import { t } from '../lib/i18n.ts';

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
  const downloads = getAllDownloads();

  return (
    <div class="stack">
      <Card title={t('downloads.heading')}>
        <p class="text-fg-muted">{t('downloads.body')}</p>
      </Card>

      <Card title={t('downloads.installers')} class="downloads">
        <p class="text-fg-muted downloads__note">{t('downloads.httpsNote')}</p>
        <div class="downloads__list">
          <For each={downloads}>
            {(dl) => (
              <div class="downloads__item">
                <div class="downloads__info">
                  <span class="downloads__platform">{dl.platform}</span>
                  <span class="downloads__meta text-fg-muted">
                    {t('downloads.version')} {dl.version}
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

      <DownloadVerifier downloads={downloads} />

      <div class="actions">
        <A href="/">
          <Button variant="ghost">{t('downloads.backToHome')}</Button>
        </A>
      </div>
    </div>
  );
}
