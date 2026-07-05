import { Button } from '@omm/ui';
import type { JSX } from 'solid-js';
import { RealmStatusCard } from './components/RealmStatusCard.tsx';
import { config } from './config.ts';
import { t } from './lib/i18n.ts';

/** App shell: brand header, live realm status, primary calls-to-action. */
export function App(): JSX.Element {
  return (
    <main class="shell">
      <header class="hero">
        <h1 class="text-fg-strong">{t('realm.title', { name: config.realmName })}</h1>
        <p class="text-fg-muted">{t('realm.tagline')}</p>
      </header>

      <RealmStatusCard />

      <nav class="actions">
        <Button variant="primary">{t('nav.play')}</Button>
        {config.features.registrationOpen && <Button variant="ghost">{t('nav.register')}</Button>}
      </nav>
    </main>
  );
}
