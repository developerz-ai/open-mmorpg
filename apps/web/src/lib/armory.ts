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

export const CharacterStatsSchema = z.object({
  kills: z.number().int().nonnegative(),
  deaths: z.number().int().nonnegative(),
  playtimeMinutes: z.number().int().nonnegative(),
});

export const CharacterTalentSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  icon: z.string().nullable(),
  maxRank: z.number().int().positive(),
  currentRank: z.number().int().nonnegative(),
});

export const AchievementSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  completedAt: z.string().nullable(),
  points: z.number().int().nonnegative(),
});

export const ActivityEventSchema = z.object({
  id: z.string(),
  type: z.enum(['boss_kill', 'item_looted', 'level_up', 'achievement', 'login']),
  timestamp: z.string(),
  description: z.string(),
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
  stats: CharacterStatsSchema.optional(),
  talents: z.array(CharacterTalentSchema).optional(),
  achievementList: z.array(AchievementSchema).optional(),
  activity: z.array(ActivityEventSchema).optional(),
});
export type Character = z.infer<typeof CharacterSchema>;
export type CharacterStats = z.infer<typeof CharacterStatsSchema>;
export type Achievement = z.infer<typeof AchievementSchema>;
export type ActivityEvent = z.infer<typeof ActivityEventSchema>;

export const GuildMemberSchema = z.object({
  name: z.string(),
  rank: z.string(),
  level: z.number().int().positive(),
  class: z.string().optional(),
  joinedAt: z.string().optional(),
});

export const GuildSchema = z.object({
  name: z.string(),
  memberCount: z.number().int().nonnegative(),
  founded: z.string().optional(),
  faction: z.string().optional(),
  description: z.string().optional(),
  achievements: z.number().int().nonnegative().optional(),
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
