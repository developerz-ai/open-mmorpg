import type { JSX } from 'solid-js';
import { cx } from './cx.ts';

export interface SkeletonProps {
  /** Visual variant. Default: 'text'. */
  variant?: 'text' | 'circle' | 'rect';
  /** Width (CSS value). Default: 100%. */
  width?: string;
  /** Height (CSS value). Default: 1em for text, 1em for circle, 100px for rect. */
  height?: string;
  /** Number of lines to animate (text variant only). */
  lines?: number;
  class?: string;
}

/**
 * Skeleton loading placeholder — shimmer animation while content loads.
 * Use before data arrives; switch to real content when ready.
 */
export function Skeleton(props: SkeletonProps): JSX.Element {
  const [local, rest] = splitProps(props, ['variant', 'width', 'height', 'lines', 'class']);

  const variant = () => local.variant ?? 'text';

  const defaultHeight = () => {
    switch (variant()) {
      case 'circle':
      case 'rect':
        return '100px';
      default:
        return '1em';
    }
  };

  const height = () => local.height ?? defaultHeight();
  const width = () => local.width ?? '100%';

  return (
    <span
      class={cx('omm-skeleton', `omm-skeleton--${variant()}`, local.class)}
      style={{ width: width(), height: height() }}
      {...rest}
    />
  );
}

import { splitProps } from 'solid-js';

export interface SkeletonTableProps {
  /** Number of rows to show. */
  rows?: number;
  /** Number of columns to show. */
  columns?: number;
  class?: string;
}

/**
 * Table skeleton — shimmer placeholder for tabular data.
 * Matches Table component structure for seamless swap.
 */
export function SkeletonTable(props: SkeletonTableProps): JSX.Element {
  const rows = () => props.rows ?? 5;
  const columns = () => props.columns ?? 4;

  return (
    <div class={cx('omm-skeleton-table', props.class)}>
      {/* Header row */}
      <div class="omm-skeleton-table__header">
        {Array.from({ length: columns() }).map(() => (
          <Skeleton variant="text" height="1.5em" width="80px" />
        ))}
      </div>
      {/* Data rows */}
      {Array.from({ length: rows() }).map(() => (
        <div class="omm-skeleton-table__row">
          {Array.from({ length: columns() }).map(() => (
            <Skeleton variant="text" height="1em" />
          ))}
        </div>
      ))}
    </div>
  );
}

export interface SkeletonCardProps {
  /** Show avatar circle at top. */
  showAvatar?: boolean;
  /** Number of text lines. */
  lines?: number;
  class?: string;
}

/**
 * Card skeleton — shimmer placeholder for card content.
 * Matches Card component structure for seamless swap.
 */
export function SkeletonCard(props: SkeletonCardProps): JSX.Element {
  const lines = () => props.lines ?? 3;

  return (
    <div class={cx('omm-skeleton-card', props.class)}>
      {props.showAvatar && <Skeleton variant="circle" width="60px" height="60px" />}
      <div class="omm-skeleton-card__content">
        <Skeleton variant="text" width="60%" height="1.5em" />
        {Array.from({ length: lines() - 1 }).map(() => (
          <Skeleton variant="text" width="100%" />
        ))}
      </div>
    </div>
  );
}
