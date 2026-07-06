import {
  Alert,
  Badge,
  Button,
  Card,
  type Column,
  Select,
  SelectOption,
  Table,
  TextField,
} from '@omm/ui';
import { A, useParams } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { createMemo, createSignal, Match, Show, Switch } from 'solid-js';
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
  {
    key: 'class',
    header: t('armory.guild.class'),
    cell: (r) => r.class ?? '—',
  },
  {
    key: 'joined',
    header: t('armory.guild.joined'),
    cell: (r) => (r.joinedAt ? fmt.date(new Date(r.joinedAt)) : '—'),
  },
];

/** Public guild roster — a read-only worldsvc projection with member links. */
export default function GuildPage(): JSX.Element {
  const params = useParams();
  const query = useGuild(() => params.name ?? '');
  const [searchQuery, setSearchQuery] = createSignal('');
  const [rankFilter, setRankFilter] = createSignal<string>('all');
  const [page, setPage] = createSignal(1);

  const MEMBERS_PER_PAGE = 25;

  // Reset page on filter change
  createMemo(() => {
    setPage(1);
  });

  // Extract unique ranks from the member list
  const availableRanks = createMemo(() => {
    if (!query.data) return [];
    const ranks = new Set(query.data.members.map((m) => m.rank));
    return Array.from(ranks).sort();
  });

  // Filter members by search query and rank
  const filteredMembers = createMemo(() => {
    if (!query.data) return [];
    const query2 = searchQuery().toLowerCase();
    const rank = rankFilter();

    return query.data.members.filter((m) => {
      const matchesSearch =
        query2 === '' ||
        m.name.toLowerCase().includes(query2) ||
        m.rank.toLowerCase().includes(query2);
      const matchesRank = rank === 'all' || m.rank === rank;
      return matchesSearch && matchesRank;
    });
  });

  // Pagination
  const totalPages = createMemo(() => Math.ceil(filteredMembers().length / MEMBERS_PER_PAGE) || 1);
  const paginatedMembers = createMemo(() => {
    const start = (page() - 1) * MEMBERS_PER_PAGE;
    return filteredMembers().slice(start, start + MEMBERS_PER_PAGE);
  });

  const paginationInfo = createMemo(() => {
    const total = filteredMembers().length;
    const start = total === 0 ? 0 : (page() - 1) * MEMBERS_PER_PAGE + 1;
    const end = Math.min(page() * MEMBERS_PER_PAGE, total);
    return { start, end, total };
  });

  return (
    <Switch>
      <Match when={query.isPending}>
        <div class="stack">
          <Card title={t('armory.heading')}>
            <div class="spinner-wrapper">
              <div class="spinner spinner--md" />
            </div>
          </Card>
        </div>
      </Match>
      <Match when={query.isError}>
        <Alert tone="info">{t('armory.notFound')}</Alert>
      </Match>
      <Match when={query.data}>
        {(g) => (
          <div class="stack">
            <Card title={g().name}>
              <div class="guild-info">
                <p class="text-fg-muted">
                  {t('armory.guild.members', { count: fmt.integer(g().memberCount) })}
                </p>
                <Show when={g().founded != null}>
                  <p class="text-fg-muted">
                    {t('armory.guild.founded')} {fmt.date(new Date(g().founded!))}
                  </p>
                </Show>
                <Show when={g().faction != null}>
                  <p class="text-fg-muted">
                    {t('armory.guild.faction')} {g().faction!}
                  </p>
                </Show>
                <Show when={g().description != null}>
                  <p class="text-fg-muted">{g().description!}</p>
                </Show>
                <Show when={g().achievements != null}>
                  <p class="actions">
                    <Badge tone="neutral">
                      {t('armory.character.achievements')}: {fmt.integer(g().achievements!)}
                    </Badge>
                  </p>
                </Show>
              </div>
            </Card>

            {/* Member search and filters */}
            <Card title={t('armory.guild.roster')}>
              <div class="toolbar">
                <TextField
                  id="member-search"
                  label={t('armory.guild.searchMembers')}
                  value={searchQuery()}
                  onInput={(e) => setSearchQuery(e.currentTarget.value)}
                  placeholder={t('armory.guild.searchPlaceholder')}
                />
                <Select
                  id="rank-filter"
                  label={t('armory.guild.filterByRank')}
                  value={rankFilter()}
                  onChange={(v) => setRankFilter(v)}
                >
                  <SelectOption value="all">{t('armory.guild.allRanks')}</SelectOption>
                  {availableRanks().map((rank) => (
                    <SelectOption value={rank}>{rank}</SelectOption>
                  ))}
                </Select>
              </div>

              <Show
                when={paginatedMembers().length > 0}
                fallback={<p class="text-fg-muted">{t('armory.guild.noMembers')}</p>}
              >
                <Table columns={columns} rows={paginatedMembers()} rowKey={(r) => r.name} />

                {/* Pagination controls */}
                <div class="pagination">
                  <Button
                    variant="ghost"
                    disabled={page() === 1}
                    onClick={() => setPage((p) => Math.max(1, p - 1))}
                  >
                    {t('feed.pagination.prev')}
                  </Button>
                  <span class="pagination-info">
                    {t('feed.pagination.page', { current: page(), total: totalPages() })}
                  </span>
                  <Button
                    variant="ghost"
                    disabled={page() === totalPages()}
                    onClick={() => setPage((p) => Math.min(totalPages(), p + 1))}
                  >
                    {t('feed.pagination.next')}
                  </Button>
                  <span class="pagination-info text-fg-muted">
                    {t('feed.pagination.showing', paginationInfo())}
                  </span>
                </div>
              </Show>
            </Card>
          </div>
        )}
      </Match>
    </Switch>
  );
}
