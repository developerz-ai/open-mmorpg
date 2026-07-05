import { describe, expect, test } from 'bun:test';
import { handleMock, MockNotFound, pattern } from './backend.ts';

describe('mock backend', () => {
  test('pattern matches a static path', () => {
    expect(pattern('/realm/status')('/realm/status')).toEqual({});
    expect(pattern('/realm/status')('/realm/other')).toBeNull();
  });

  test('pattern extracts and decodes path params', () => {
    const match = pattern('/armory/character/:name')('/armory/character/Sir%20Aria');
    expect(match).toEqual({ name: 'Sir Aria' });
  });

  test('an unregistered route throws MockNotFound', async () => {
    await expect(handleMock('gateway', 'GET', '/nope', undefined)).rejects.toBeInstanceOf(
      MockNotFound,
    );
  });
});
