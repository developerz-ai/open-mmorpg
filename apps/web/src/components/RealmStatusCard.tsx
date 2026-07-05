import type { JSX } from 'solid-js';
import { Match, Switch } from 'solid-js';
import { t } from '../lib/i18n.ts';
import { useRealmStatus } from '../queries/useRealmStatus.ts';

/**
 * Live realm status. SRP: this component only renders the query's three states
 * (loading / error / data). The fetch and validation live behind the hook.
 */
export function RealmStatusCard(): JSX.Element {
  const status = useRealmStatus();
  return (
    <section class="card">
      <h2 class="text-fg-strong">{t('realm.status.heading')}</h2>
      <Switch>
        <Match when={status.isPending}>
          <p class="text-fg-muted">{t('realm.status.loading')}</p>
        </Match>
        <Match when={status.isError}>
          <p class="text-fg-muted">{t('realm.status.error')}</p>
        </Match>
        <Match when={status.data}>
          {(data) => (
            <>
              <p class="badge">
                {data().online ? t('realm.status.online') : t('realm.status.offline')}
              </p>
              <p class="text-fg-muted">
                {t('realm.status.population', {
                  count: data().population,
                  capacity: data().capacity,
                })}
              </p>
            </>
          )}
        </Match>
      </Switch>
    </section>
  );
}
