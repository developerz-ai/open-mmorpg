import { Badge, Button, Card, Spinner, TextField } from '@omm/ui';
import type { JSX } from 'solid-js';
import { createSignal, Show } from 'solid-js';
import { t } from '../lib/i18n.ts';

export interface ChecksumVerifierProps {
  /** Expected checksum to validate against */
  expectedChecksum: string;
}

/**
 * Checksum verification UI. User enters their computed checksum; validates against
 * expected and shows verified/unverified status via Badge.
 * → docs/specs/web-client/downloads#verification
 */
export function ChecksumVerifier(props: ChecksumVerifierProps): JSX.Element {
  const [input, setInput] = createSignal('');
  const [status, setStatus] = createSignal<'idle' | 'verifying' | 'match' | 'mismatch' | 'invalid'>(
    'idle',
  );

  const isValidHex = (str: string): boolean => {
    return /^[a-f0-9]{64}$/i.test(str.trim());
  };

  const handleVerify = (): void => {
    const value = input().trim();

    if (!isValidHex(value)) {
      setStatus('invalid');
      return;
    }

    setStatus('verifying');

    // Simulate async verification (in real impl, might hash large file)
    setTimeout(() => {
      if (value.toLowerCase() === props.expectedChecksum.toLowerCase()) {
        setStatus('match');
      } else {
        setStatus('mismatch');
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

      <Show when={status() === 'match'}>
        <div class="verifier__result">
          <Badge tone="success">{t('downloads.verifier.match')}</Badge>
        </div>
      </Show>

      <Show when={status() === 'mismatch'}>
        <div class="verifier__result">
          <Badge tone="danger">{t('downloads.verifier.mismatch')}</Badge>
        </div>
      </Show>
    </Card>
  );
}
