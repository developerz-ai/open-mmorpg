import { z } from 'zod';
import { config } from '../config.ts';

/**
 * Zod schema for the gateway's `/realm/status`. We validate at the boundary —
 * never trust the shape of a network response — and the inferred type flows into
 * the UI. This mirrors the gateway's `RealmStatus` (crates-side source of truth).
 */
export const RealmStatusSchema = z.object({
  name: z.string(),
  online: z.boolean(),
  population: z.number().int().nonnegative(),
  capacity: z.number().int().positive(),
});

export type RealmStatus = z.infer<typeof RealmStatusSchema>;

/** Fetch and validate realm status from the operator's gateway. */
export async function fetchRealmStatus(): Promise<RealmStatus> {
  const res = await fetch(`${config.gatewayUrl}/realm/status`);
  if (!res.ok) throw new Error(`realm status ${res.status}`);
  return RealmStatusSchema.parse(await res.json());
}
