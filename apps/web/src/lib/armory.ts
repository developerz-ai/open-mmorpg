import { z } from 'zod';
import { request } from './api.ts';

/**
 * Armory projections — **read-only** views served by worldsvc. The web computes
 * nothing about game state; it Zod-parses the projection and renders it. A shape
 * drift fails loud. → docs/specs/web-client/armory
 */
export const GearSlotSchema = z.object({
  slot: z.string(),
  item: z.string(),
  itemLevel: z.number().int().nonnegative(),
});

export const CharacterSchema = z.object({
  name: z.string(),
  race: z.string(),
  class: z.string(),
  level: z.number().int().positive(),
  itemLevel: z.number().int().nonnegative(),
  guild: z.string().nullable(),
  achievements: z.number().int().nonnegative(),
  gear: z.array(GearSlotSchema),
});
export type Character = z.infer<typeof CharacterSchema>;

export const GuildMemberSchema = z.object({
  name: z.string(),
  rank: z.string(),
  level: z.number().int().positive(),
});

export const GuildSchema = z.object({
  name: z.string(),
  memberCount: z.number().int().nonnegative(),
  members: z.array(GuildMemberSchema),
});
export type Guild = z.infer<typeof GuildSchema>;

/** Fetch a character projection by name. */
export function fetchCharacter(name: string): Promise<Character> {
  return request({
    backend: 'worldsvc',
    path: `/armory/character/${encodeURIComponent(name)}`,
    schema: CharacterSchema,
  });
}

/** Fetch a guild projection by name. */
export function fetchGuild(name: string): Promise<Guild> {
  return request({
    backend: 'worldsvc',
    path: `/armory/guild/${encodeURIComponent(name)}`,
    schema: GuildSchema,
  });
}
