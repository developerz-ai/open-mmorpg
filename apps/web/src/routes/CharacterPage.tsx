import { Alert, Badge, Card, type Column, Spinner, Table } from '@omm/ui';
import { useParams } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { Match, Show, Switch } from 'solid-js';
import type { Character } from '../lib/armory.ts';
import { fmt, t } from '../lib/i18n.ts';
import { useCharacter } from '../queries/useArmory.ts';

type GearRow = Character['gear'][number];

const gearColumns: Column<GearRow>[] = [
  { key: 'slot', header: t('armory.character.slot'), cell: (r) => r.slot },
  { key: 'item', header: t('armory.character.item'), cell: (r) => r.item },
  {
    key: 'ilvl',
    header: t('armory.character.itemLevelCol'),
    cell: (r) => fmt.integer(r.itemLevel),
  },
];

/** Public character sheet — a read-only worldsvc projection. */
export default function CharacterPage(): JSX.Element {
  const params = useParams();
  const query = useCharacter(() => params.name ?? '');

  return (
    <Switch>
      <Match when={query.isPending}>
        <Spinner label={t('common.loading')} />
      </Match>
      <Match when={query.isError}>
        <Alert tone="info">{t('armory.notFound')}</Alert>
      </Match>
      <Match when={query.data}>
        {(c) => (
          <div class="stack">
            <Card>
              <h1 class="text-fg-strong">{c().name}</h1>
              <p class="text-fg-muted">
                {t('armory.character.level', {
                  level: fmt.integer(c().level),
                  race: c().race,
                  class: c().class,
                })}
              </p>
              <p class="actions">
                <Badge tone="accent">
                  {t('armory.character.itemLevel', { value: fmt.integer(c().itemLevel) })}
                </Badge>
                <Show when={c().guild}>{(g) => <Badge tone="neutral">{g()}</Badge>}</Show>
                <Badge tone="neutral">
                  {t('armory.character.achievements')}: {fmt.integer(c().achievements)}
                </Badge>
              </p>
            </Card>
            <Card title={t('armory.character.gear')}>
              <Show
                when={c().gear.length > 0}
                fallback={<p class="text-fg-muted">{t('armory.character.empty')}</p>}
              >
                <Table columns={gearColumns} rows={c().gear} rowKey={(r) => r.slot} />
              </Show>
            </Card>
          </div>
        )}
      </Match>
    </Switch>
  );
}
