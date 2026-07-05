/**
 * `Intl`-based formatting — the boundary between *copy* (catalogs) and
 * *formatting* (locale-aware numbers, dates, money). Catalogs hold words;
 * these hold shape. A new locale gets correct dates and grouping for free.
 *
 * Formatters are built once per locale (constructing `Intl.*Format` is not
 * free) and returned as a bundle so callers never hand-write a format string.
 */
import type { Locale } from './locales.ts';

/** Locale-aware formatters for one locale. */
export interface Formatters {
  /** Grouped integer, e.g. `1,204`. */
  integer(value: number): string;
  /** Decimal with up to 2 fraction digits. */
  decimal(value: number): string;
  /** Compact count, e.g. `12.4K`. */
  compact(value: number): string;
  /** In-game gold — grouped integer suffixed by the caller's `t('…gold')`. */
  gold(value: number): string;
  /** Absolute date+time, medium form. */
  dateTime(value: Date | number): string;
  /** Absolute date only, medium form. */
  date(value: Date | number): string;
  /** Relative time from `now` (default: the passed clock is required by tests). */
  relative(value: Date | number, now: Date | number): string;
}

const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ['year', 31_536_000_000],
  ['month', 2_592_000_000],
  ['day', 86_400_000],
  ['hour', 3_600_000],
  ['minute', 60_000],
  ['second', 1_000],
];

/** Build the formatter bundle for a locale. Pure given `Intl` data. */
export function createFormatters(locale: Locale): Formatters {
  const integer = new Intl.NumberFormat(locale, { maximumFractionDigits: 0 });
  const decimal = new Intl.NumberFormat(locale, { maximumFractionDigits: 2 });
  const compact = new Intl.NumberFormat(locale, { notation: 'compact', maximumFractionDigits: 1 });
  const dateTime = new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' });
  const date = new Intl.DateTimeFormat(locale, { dateStyle: 'medium' });
  const relative = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' });

  return {
    integer: (v) => integer.format(v),
    decimal: (v) => decimal.format(v),
    compact: (v) => compact.format(v),
    gold: (v) => integer.format(Math.trunc(v)),
    dateTime: (v) => dateTime.format(v),
    date: (v) => date.format(v),
    relative: (v, now) => {
      const diff = toMs(v) - toMs(now);
      const abs = Math.abs(diff);
      for (const [unit, ms] of UNITS) {
        if (abs >= ms || unit === 'second') {
          return relative.format(Math.round(diff / ms), unit);
        }
      }
      return relative.format(0, 'second');
    },
  };
}

function toMs(value: Date | number): number {
  return typeof value === 'number' ? value : value.getTime();
}
