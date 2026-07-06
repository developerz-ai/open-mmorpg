import { useQuery } from '@tanstack/solid-query';
import { type Character, fetchCharacter, fetchGuild, type Guild } from '../lib/armory.ts';

/** Character projection, keyed by name. Public data → cached, no polling. */
export function useCharacter(name: () => string) {
  return useQuery<Character>(() => ({
    queryKey: ['armory', 'character', name()],
    queryFn: () => fetchCharacter(name()),
    staleTime: 60_000,
    retry: 2,
    retryDelay: (attempt) => Math.min(1000 * 2 ** (attempt - 1), 10000),
  }));
}

/** Guild projection, keyed by name. */
export function useGuild(name: () => string) {
  return useQuery<Guild>(() => ({
    queryKey: ['armory', 'guild', name()],
    queryFn: () => fetchGuild(name()),
    staleTime: 60_000,
    retry: 2,
    retryDelay: (attempt) => Math.min(1000 * 2 ** (attempt - 1), 10000),
  }));
}
