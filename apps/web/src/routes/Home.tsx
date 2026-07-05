import { Button, Card } from '@omm/ui';
import { A } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { For, Show } from 'solid-js';
import { RealmStatusCard } from '../components/RealmStatusCard.tsx';
import { config } from '../config.ts';
import { isEnabled } from '../lib/features.ts';
import { t } from '../lib/i18n.ts';

interface Feature {
  key: string;
  title: string;
  body: string;
}

const FEATURES: Feature[] = [
  { key: 'world', title: 'home.feature.worldTitle', body: 'home.feature.worldBody' },
  { key: 'economy', title: 'home.feature.economyTitle', body: 'home.feature.economyBody' },
  { key: 'armory', title: 'home.feature.armoryTitle', body: 'home.feature.armoryBody' },
];

/**
 * Marketing landing — hero, live realm status, download CTA, feature grid.
 * Thin: it composes `t()` copy and `packages/ui` primitives; no data logic of
 * its own (realm status lives behind its own hook/component).
 */
export function Home(): JSX.Element {
  return (
    <div class="stack">
      <section class="hero">
        <h1 class="text-fg-strong">{t('realm.title', { name: config.brand.realmName })}</h1>
        <p class="text-fg-muted hero__tagline">{config.brand.tagline ?? t('realm.tagline')}</p>
        <nav class="actions" aria-label={t('home.heroCta')}>
          <A href="#download">
            <Button variant="primary">{t('home.heroCta')}</Button>
          </A>
          <Show when={isEnabled('registrationOpen')}>
            <A href="/register">
              <Button variant="ghost">{t('nav.register')}</Button>
            </A>
          </Show>
        </nav>
      </section>

      <RealmStatusCard />

      <section class="feature-grid">
        <For each={FEATURES}>
          {(f) => (
            <Card title={t(f.title)}>
              <p class="text-fg-muted">{t(f.body)}</p>
            </Card>
          )}
        </For>
      </section>

      <Card title={t('home.downloadHeading')} class="download">
        <p class="text-fg-muted">{t('home.downloadBody')}</p>
        <div id="download" class="actions">
          <Button variant="primary">{t('home.download')}</Button>
        </div>
      </Card>
    </div>
  );
}
