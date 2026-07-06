import { Card, Skeleton, SkeletonTable } from '@omm/ui';
import type { JSX } from 'solid-js';

/**
 * Skeleton loading state for AuctionHouse page.
 * Matches the full layout: filters toolbar + table + pagination.
 */
export function SkeletonAuctionHouse(): JSX.Element {
  return (
    <div class="stack">
      <Card title="Loading...">
        <div class="toolbar">
          {/* Search field skeleton */}
          <Skeleton variant="rect" width="200px" height="40px" />
          {/* Category filter skeleton */}
          <Skeleton variant="rect" width="150px" height="40px" />
          {/* Price range skeletons */}
          <Skeleton variant="rect" width="100px" height="40px" />
          <Skeleton variant="rect" width="100px" height="40px" />
        </div>
      </Card>

      {/* Listings table skeleton */}
      <SkeletonTable rows={10} columns={6} />

      {/* Pagination skeleton */}
      <div class="pagination">
        <Skeleton variant="rect" width="200px" height="32px" />
      </div>
    </div>
  );
}
