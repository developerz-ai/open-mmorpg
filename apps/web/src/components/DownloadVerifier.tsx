import { Badge, Button, Card, Spinner, TextField } from '@omm/ui';
import type { JSX } from 'solid-js';
import { createSignal, For, Show } from 'solid-js';
import type { PlatformDownload } from '../lib/downloads.ts';
import { t } from '../lib/i18n.ts';

export interface DownloadVerifierProps {
  /** All available platform downloads with their checksums */
  downloads: PlatformDownload[];
}

/**
 * Multi-download checksum verifier. User enters their computed checksum;
 * validates against all available downloads and shows which one matches.
 * → docs/specs/web-client/downloads#verification
 */
export function DownloadVerifier(props: DownloadVerifierProps): JSX.Element {
  const [input, setInput] = createSignal('');
  const [status, setStatus] = createSignal<'idle' | 'verifying' | 'match' | 'mismatch' | 'invalid'>(
    'idle',
  );
  const [matchedDownload, setMatchedDownload] = createSignal<PlatformDownload | null>(null);

  const isValidHex = (str: string): boolean => {
    return /^[a-f0-9]{64}$/i.test(str.trim());
  };

  const handleVerify = (): void => {
    const value = input().trim();

    if (!isValidHex(value)) {
      setStatus('invalid');
      setMatchedDownload(null);
      return;
    }

    setStatus('verifying');

    // Simulate async verification (in real impl, might hash large file)
    setTimeout(() => {
      const match = props.downloads.find((dl) => dl.checksum.toLowerCase() === value.toLowerCase());

      if (match) {
        setStatus('match');
        setMatchedDownload(match);
      } else {
        setStatus('mismatch');
        setMatchedDownload(null);
      }
    }, 300);
  };

  const handleKeyDown = (e: KeyboardEvent): void => {
    if (e.key === 'Enter' && status() !== 'verifying') {
      handleVerify();
    }
  };

  return (
    <Card title={t('downloads.verifier.heading')} class="verifier">
      <TextField
        id="checksum-input"
        label={t('downloads.verifier.label')}
        placeholder={t('downloads.verifier.placeholder')}
        value={input()}
        onInput={(e) => {
          setInput(e.currentTarget.value);
          setStatus('idle');
          setMatchedDownload(null);
        }}
        onKeyDown={handleKeyDown}
        error={status() === 'invalid' ? t('downloads.verifier.invalid') : undefined}
      />

      <div class="verifier__actions">
        <Button
          variant="primary"
          onClick={handleVerify}
          disabled={status() === 'verifying' || input().trim() === ''}
        >
          <Show when={status() === 'verifying'} fallback={t('downloads.verifier.verify')}>
            <Spinner label={t('downloads.verifier.verifying')} size="sm" class="btn-spinner" />
            <span class="btn-text">{t('downloads.verifier.verifying')}</span>
          </Show>
        </Button>
      </div>

      <Show when={status() === 'match' && matchedDownload()}>
        <div class="verifier__result">
          <Badge tone="success">{t('downloads.verifier.match')}</Badge>
          <div class="verifier__match">
            <span class="text-fg-muted">
              {matchedDownload()?.platform} {matchedDownload()?.version}
            </span>
          </div>
        </div>
      </Show>

      <Show when={status() === 'mismatch'}>
        <div class="verifier__result">
          <Badge tone="danger">{t('downloads.verifier.mismatch')}</Badge>
        </div>
      </Show>

      <div class="verifier__help">
        <p class="text-fg-muted">{t('downloads.verificationBody')}</p>
        <For each={props.downloads.slice(0, 3)}>
          {(dl) => (
            <div class="verifier__example">
              <span class="verifier__platform text-fg-muted">{dl.platform}:</span>
              <code class="verifier__checksum">{dl.checksum}</code>
            </div>
          )}
        </For>
        {props.downloads.length > 3 && (
          <div class="verifier__more text-fg-muted">
            +{props.downloads.length - 3} more platforms
          </div>
        )}
      </div>
    </Card>
  );
}
