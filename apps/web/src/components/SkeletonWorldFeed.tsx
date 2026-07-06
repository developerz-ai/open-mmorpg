import { Card, Skeleton, SkeletonCard } from '@omm/ui';
import type { JSX } from 'solid-js';

/**
 * Skeleton loading state for WorldFeed page.
 * Matches the full layout: filters toolbar + feed items + pagination.
 */
export function SkeletonWorldFeed(): JSX.Element {
  return (
    <div class="stack">
      <Card title="Loading...">
        <div class="toolbar">
          {/* Event type filter skeleton */}
          <Skeleton variant="rect" width="150px" height="40px" />
          {/* Search skeleton */}
          <Skeleton variant="rect" width="200px" height="40px" />
        </div>
      </Card>

      {/* Feed item skeletons */}
      {Array.from({ length: 8 }).map(() => (
        <SkeletonCard lines={2} />
      ))}

      {/* Pagination skeleton */}
      <div class="pagination">
        <Skeleton variant="rect" width="200px" height="32px" />
      </div>
    </div>
  );
}
