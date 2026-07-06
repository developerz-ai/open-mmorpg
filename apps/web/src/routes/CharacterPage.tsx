import { Alert, Badge, Button, Card, type Column, Progress, Spinner, Table } from '@omm/ui';
import { useParams } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { createMemo, createSignal, Match, Show, Switch } from 'solid-js';
import type { Achievement, ActivityEvent, Character, CharacterStats } from '../lib/armory.ts';
import { fmt, t } from '../lib/i18n.ts';
import { useCharacter } from '../queries/useArmory.ts';

type GearRow = Character['gear'][number];
type AchievementRow = Achievement;
type ActivityRow = ActivityEvent;

const gearColumns: Column<GearRow>[] = [
  { key: 'slot', header: t('armory.character.slot'), cell: (r) => r.slot },
  { key: 'item', header: t('armory.character.item'), cell: (r) => r.item },
  {
    key: 'ilvl',
    header: t('armory.character.itemLevelCol'),
    cell: (r) => fmt.integer(r.itemLevel),
  },
];

const achievementColumns: Column<AchievementRow>[] = [
  { key: 'name', header: t('armory.character.achievement'), cell: (r) => r.name },
  {
    key: 'completed',
    header: t('armory.character.completed'),
    cell: (r) => (r.completedAt ? fmt.date(new Date(r.completedAt)) : '—'),
  },
  {
    key: 'points',
    header: t('armory.character.points'),
    cell: (r) => fmt.integer(r.points),
  },
];

const activityColumns: Column<ActivityRow>[] = [
  {
    key: 'time',
    header: t('armory.character.time'),
    cell: (r) => fmt.date(new Date(r.timestamp)),
  },
  {
    key: 'type',
    header: t('armory.character.type'),
    cell: (r) => <Badge tone={r.type === 'achievement' ? 'success' : 'neutral'}>{r.type}</Badge>,
  },
  { key: 'description', header: t('armory.character.event'), cell: (r) => r.description },
];

/** Stats panel component */
function StatsPanel(props: { stats: CharacterStats }): JSX.Element {
  const kdRatio =
    props.stats.deaths > 0 ? (props.stats.kills / props.stats.deaths).toFixed(2) : '—';

  const hours = Math.floor(props.stats.playtimeMinutes / 60);
  const minutes = props.stats.playtimeMinutes % 60;

  return (
    <Card title={t('armory.character.stats')}>
      <div class="stats-grid">
        <div class="stat-item">
          <span class="stat-label">{t('armory.character.kills')}</span>
          <span class="stat-value">{fmt.integer(props.stats.kills)}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">{t('armory.character.deaths')}</span>
          <span class="stat-value">{fmt.integer(props.stats.deaths)}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">{t('armory.character.kdRatio')}</span>
          <span class="stat-value">{kdRatio}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">{t('armory.character.playtime')}</span>
          <span class="stat-value">{hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`}</span>
        </div>
      </div>
    </Card>
  );
}

/** Talents panel component */
function TalentsPanel(props: { talents: Character['talents'] }): JSX.Element {
  if (!props.talents || props.talents.length === 0) {
    return null;
  }

  return (
    <Card title={t('armory.character.talents')}>
      <div class="talents-grid">
        {props.talents.map((talent) => (
          <div class="talent-item" classList={{ 'talent-item': true }} data-id={talent.id}>
            <Show when={talent.icon}>
              {(icon) => (
                <div class="talent-icon">
                  <img src={icon()} alt={talent.name} />
                </div>
              )}
            </Show>
            <div class="talent-info">
              <div class="talent-name">{talent.name}</div>
              <div class="talent-rank">
                {fmt.integer(talent.currentRank)} / {fmt.integer(talent.maxRank)}
              </div>
              <Progress value={(talent.currentRank / talent.maxRank) * 100} label="" />
            </div>
          </div>
        ))}
      </div>
    </Card>
  );
}

/** Public character sheet — a read-only worldsvc projection. */
export default function CharacterPage(): JSX.Element {
  const params = useParams();
  const query = useCharacter(() => params.name ?? '');
  const [achievementsPage, setAchievementsPage] = createSignal(1);
  const [activityPage, setActivityPage] = createSignal(1);

  const ACHIEVEMENTS_PER_PAGE = 10;
  const ACTIVITY_PER_PAGE = 15;

  const paginatedAchievements = createMemo(() => {
    const list = query.data?.achievementList;
    if (!list || list.length === 0) return [];
    const start = (achievementsPage() - 1) * ACHIEVEMENTS_PER_PAGE;
    return list.slice(start, start + ACHIEVEMENTS_PER_PAGE);
  });

  const achievementsTotalPages = createMemo(() => {
    const list = query.data?.achievementList;
    if (!list || list.length === 0) return 1;
    return Math.ceil(list.length / ACHIEVEMENTS_PER_PAGE);
  });

  const paginatedActivity = createMemo(() => {
    const list = query.data?.activity;
    if (!list || list.length === 0) return [];
    const start = (activityPage() - 1) * ACTIVITY_PER_PAGE;
    return list.slice(start, start + ACTIVITY_PER_PAGE);
  });

  const activityTotalPages = createMemo(() => {
    const list = query.data?.activity;
    if (!list || list.length === 0) return 1;
    return Math.ceil(list.length / ACTIVITY_PER_PAGE);
  });

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

            {/* Stats Panel */}
            <Show when={c().stats}>{(stats) => <StatsPanel stats={stats()} />}</Show>

            {/* Talents Panel */}
            <Show
              when={(() => {
                const talents = c().talents;
                return talents != null && talents.length > 0;
              })()}
            >
              <TalentsPanel talents={c().talents!} />
            </Show>

            {/* Gear */}
            <Card title={t('armory.character.gear')}>
              <Show
                when={c().gear.length > 0}
                fallback={<p class="text-fg-muted">{t('armory.character.empty')}</p>}
              >
                <Table columns={gearColumns} rows={c().gear} rowKey={(r) => r.slot} />
              </Show>
            </Card>

            {/* Achievements */}
            <Show
              when={(() => {
                const list = c().achievementList;
                return list != null && list.length > 0;
              })()}
            >
              <Card title={t('armory.character.achievementsList')}>
                <Table
                  columns={achievementColumns}
                  rows={paginatedAchievements()}
                  rowKey={(r) => r.id}
                />
                <div class="pagination">
                  <Button
                    variant="ghost"
                    disabled={achievementsPage() === 1}
                    onClick={() => setAchievementsPage((p) => Math.max(1, p - 1))}
                  >
                    {t('feed.pagination.prev')}
                  </Button>
                  <span class="pagination-info">
                    {t('feed.pagination.page', {
                      current: achievementsPage(),
                      total: achievementsTotalPages(),
                    })}
                  </span>
                  <Button
                    variant="ghost"
                    disabled={achievementsPage() === achievementsTotalPages()}
                    onClick={() =>
                      setAchievementsPage((p) => Math.min(achievementsTotalPages(), p + 1))
                    }
                  >
                    {t('feed.pagination.next')}
                  </Button>
                </div>
              </Card>
            </Show>

            {/* Activity Timeline */}
            <Show
              when={(() => {
                const activity = c().activity;
                return activity != null && activity.length > 0;
              })()}
            >
              <Card title={t('armory.character.activity')}>
                <Table columns={activityColumns} rows={paginatedActivity()} rowKey={(r) => r.id} />
                <div class="pagination">
                  <Button
                    variant="ghost"
                    disabled={activityPage() === 1}
                    onClick={() => setActivityPage((p) => Math.max(1, p - 1))}
                  >
                    {t('feed.pagination.prev')}
                  </Button>
                  <span class="pagination-info">
                    {t('feed.pagination.page', {
                      current: activityPage(),
                      total: activityTotalPages(),
                    })}
                  </span>
                  <Button
                    variant="ghost"
                    disabled={activityPage() === activityTotalPages()}
                    onClick={() => setActivityPage((p) => Math.min(activityTotalPages(), p + 1))}
                  >
                    {t('feed.pagination.next')}
                  </Button>
                </div>
              </Card>
            </Show>
          </div>
        )}
      </Match>
    </Switch>
  );
}
