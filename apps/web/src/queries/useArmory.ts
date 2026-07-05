import { useQuery } from '@tanstack/solid-query';
import { type Character, fetchCharacter, fetchGuild, type Guild } from '../lib/armory.ts';

/** Character projection, keyed by name. Public data → cached, no polling. */
export function useCharacter(name: () => string) {
  return useQuery<Character>(() => ({
    queryKey: ['armory', 'character', name()],
    queryFn: () => fetchCharacter(name()),
    staleTime: 60_000,
    retry: false,
  }));
}

/** Guild projection, keyed by name. */
export function useGuild(name: () => string) {
  return useQuery<Guild>(() => ({
    queryKey: ['armory', 'guild', name()],
    queryFn: () => fetchGuild(name()),
    staleTime: 60_000,
    retry: false,
  }));
}
