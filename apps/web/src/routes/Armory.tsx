import { Button, Card, TextField } from '@omm/ui';
import { useNavigate } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { createSignal } from 'solid-js';
import { t } from '../lib/i18n.ts';

/** Armory search — thin entry point that routes to a character or guild page. */
export default function Armory(): JSX.Element {
  const navigate = useNavigate();
  const [name, setName] = createSignal('');
  const go = (kind: 'character' | 'guild'): void => {
    const q = name().trim();
    if (q) navigate(`/armory/${kind}/${encodeURIComponent(q)}`);
  };

  return (
    <Card title={t('armory.heading')} class="stack">
      <div class="toolbar">
        <TextField
          id="armory-search"
          label={t('armory.searchLabel')}
          placeholder={t('armory.searchPlaceholder')}
          value={name()}
          onInput={(e) => setName(e.currentTarget.value)}
          onKeyDown={(e) => e.key === 'Enter' && go('character')}
        />
        <Button variant="primary" onClick={() => go('character')}>
          {t('armory.searchCharacter')}
        </Button>
        <Button variant="ghost" onClick={() => go('guild')}>
          {t('armory.searchGuild')}
        </Button>
      </div>
    </Card>
  );
}
