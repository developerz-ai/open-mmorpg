import { z } from 'zod';
import { request } from './api.ts';

/**
 * World feed — a **read-only projection** of the living-world event stream. Each
 * event is parsed into a typed, discriminated union; an **unknown variant
 * degrades gracefully** (rendered as a neutral line) rather than crashing the
 * stream. → docs/specs/web-client/world-feed
 */
const base = { id: z.string(), at: z.string() };

export const FeedItemSchema = z.discriminatedUnion('kind', [
  z.object({ ...base, kind: z.literal('boss_kill'), actor: z.string(), target: z.string() }),
  z.object({ ...base, kind: z.literal('world_boss_spawn'), target: z.string(), zone: z.string() }),
  z.object({ ...base, kind: z.literal('faction_shift'), actor: z.string(), zone: z.string() }),
  z.object({ ...base, kind: z.literal('bounty_posted'), target: z.string() }),
  z.object({ ...base, kind: z.literal('milestone'), message: z.string() }),
]);
export type FeedItem = z.infer<typeof FeedItemSchema>;

/** An event whose variant we don't recognise — kept, rendered neutrally. */
export interface UnknownFeedItem {
  kind: 'unknown';
  id: string;
  at: string;
}
export type FeedEntry = FeedItem | UnknownFeedItem;

const RawFeedSchema = z.array(z.unknown());

/** Extract a stable id + timestamp from an unrecognised event, best-effort. */
function toUnknown(raw: unknown, index: number): UnknownFeedItem {
  const obj = (raw ?? {}) as { id?: unknown; at?: unknown };
  return {
    kind: 'unknown',
    id: typeof obj.id === 'string' ? obj.id : `unknown-${index}`,
    at: typeof obj.at === 'string' ? obj.at : '',
  };
}

/**
 * Fetch the world feed. We parse each item independently so one malformed or
 * newer-than-us event degrades to `unknown` instead of failing the whole stream.
 */
export async function fetchFeed(): Promise<FeedEntry[]> {
  const raw = await request({ backend: 'worldsvc', path: '/world/feed', schema: RawFeedSchema });
  return raw.map((item, i) => {
    const parsed = FeedItemSchema.safeParse(item);
    return parsed.success ? parsed.data : toUnknown(item, i);
  });
}
