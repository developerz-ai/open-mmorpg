import { describe, expect, test } from 'bun:test';
import { fetchCharacter, fetchGuild } from './armory.ts';
import { NetworkError } from './errors.ts';

describe('armory (mock worldsvc)', () => {
  test('fetches a character projection by name (case-insensitive)', async () => {
    const char = await fetchCharacter('Aria');
    expect(char.class).toBe('Warden');
    expect(char.gear.length).toBeGreaterThan(0);
    expect(char.guild).toBe('Vanguard');
  });

  test('a character with no guild parses guild as null', async () => {
    const char = await fetchCharacter('Kael');
    expect(char.guild).toBeNull();
  });

  test('an unknown character is a not-found (NetworkError)', async () => {
    await expect(fetchCharacter('Nobody')).rejects.toBeInstanceOf(NetworkError);
  });

  test('fetches a guild roster', async () => {
    const guild = await fetchGuild('Vanguard');
    expect(guild.memberCount).toBe(3);
    expect(guild.members[0]?.rank).toBe('Guildmaster');
  });
});
