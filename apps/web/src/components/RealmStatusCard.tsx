import { Badge, Card, Spinner } from '@omm/ui';
import type { JSX } from 'solid-js';
import { Match, Switch } from 'solid-js';
import { fmt, t } from '../lib/i18n.ts';
import { useRealmStatus } from '../queries/useRealmStatus.ts';

/**
 * Live realm status. SRP: this component only renders the query's three states
 * (loading / error / data). The fetch and validation live behind the hook;
 * counts format via `Intl` (`fmt`), never a hand-written string.
 */
export function RealmStatusCard(): JSX.Element {
  const status = useRealmStatus();
  return (
    <Card title={t('realm.status.heading')}>
      <Switch>
        <Match when={status.isPending}>
          <p class="text-fg-muted">
            <Spinner label={t('realm.status.loading')} /> {t('realm.status.loading')}
          </p>
        </Match>
        <Match when={status.isError}>
          <p class="text-fg-muted">{t('realm.status.error')}</p>
        </Match>
        <Match when={status.data}>
          {(data) => (
            <>
              <p>
                <Badge tone={data().online ? 'success' : 'neutral'}>
                  {data().online ? t('realm.status.online') : t('realm.status.offline')}
                </Badge>
              </p>
              <p class="text-fg-muted">
                {t('realm.status.population', {
                  count: fmt.integer(data().population),
                  capacity: fmt.integer(data().capacity),
                })}
              </p>
            </>
          )}
        </Match>
      </Switch>
    </Card>
  );
}
