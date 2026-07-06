import { Card, Skeleton, SkeletonCard, SkeletonTable } from '@omm/ui';
import type { JSX } from 'solid-js';

/**
 * Skeleton loading state for CharacterPage.
 * Matches the layout: stats panel + gear table + achievements table + activity timeline.
 */
export function SkeletonCharacterPage(): JSX.Element {
  return (
    <div class="stack">
      {/* Stats panel skeleton */}
      <Card title="Loading...">
        <div class="stats-grid">
          {Array.from({ length: 4 }).map(() => (
            <div class="stat-item">
              <Skeleton variant="text" width="80px" height="1em" />
              <Skeleton variant="text" width="60px" height="1.5em" />
            </div>
          ))}
        </div>
      </Card>

      {/* Gear table skeleton */}
      <SkeletonTable rows={10} columns={3} />

      {/* Achievements table skeleton */}
      <SkeletonTable rows={5} columns={3} />

      {/* Activity timeline skeleton */}
      <Card title="Loading...">
        {Array.from({ length: 6 }).map(() => (
          <SkeletonCard lines={2} />
        ))}
      </Card>
    </div>
  );
}
