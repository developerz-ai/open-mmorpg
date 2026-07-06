import { Card, Skeleton, SkeletonTable } from '@omm/ui';
import type { JSX } from 'solid-js';

/**
 * Skeleton loading state for GuildPage.
 * Matches the layout: search/filters toolbar + members table + pagination.
 */
export function SkeletonGuildPage(): JSX.Element {
  return (
    <div class="stack">
      <Card title="Loading...">
        <div class="toolbar">
          {/* Search field skeleton */}
          <Skeleton variant="rect" width="200px" height="40px" />
          {/* Rank filter skeleton */}
          <Skeleton variant="rect" width="150px" height="40px" />
        </div>
      </Card>

      {/* Members table skeleton */}
      <SkeletonTable rows={15} columns={5} />

      {/* Pagination skeleton */}
      <div class="pagination">
        <Skeleton variant="rect" width="200px" height="32px" />
      </div>
    </div>
  );
}
