import { Alert, Card, type Column, Spinner, Table } from '@omm/ui';
import { A, useParams } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { Match, Switch } from 'solid-js';
import type { Guild } from '../lib/armory.ts';
import { fmt, t } from '../lib/i18n.ts';
import { useGuild } from '../queries/useArmory.ts';

type MemberRow = Guild['members'][number];

const columns: Column<MemberRow>[] = [
  {
    key: 'name',
    header: t('armory.guild.member'),
    cell: (r) => <A href={`/armory/character/${encodeURIComponent(r.name)}`}>{r.name}</A>,
  },
  { key: 'rank', header: t('armory.guild.rank'), cell: (r) => r.rank },
  { key: 'level', header: t('armory.guild.level'), cell: (r) => fmt.integer(r.level) },
];

/** Public guild roster — a read-only worldsvc projection with member links. */
export default function GuildPage(): JSX.Element {
  const params = useParams();
  const query = useGuild(() => params.name ?? '');

  return (
    <Switch>
      <Match when={query.isPending}>
        <Spinner label={t('common.loading')} />
      </Match>
      <Match when={query.isError}>
        <Alert tone="info">{t('armory.notFound')}</Alert>
      </Match>
      <Match when={query.data}>
        {(g) => (
          <Card title={g().name}>
            <p class="text-fg-muted">
              {t('armory.guild.members', { count: fmt.integer(g().memberCount) })}
            </p>
            <Table columns={columns} rows={g().members} rowKey={(r) => r.name} />
          </Card>
        )}
      </Match>
    </Switch>
  );
}
