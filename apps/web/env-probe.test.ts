// TEMP probe for task tsk_dc96ae20b7ea1e11334a105972d33f31 — deleted after measurement.
import { describe, expect, test } from 'bun:test';

describe('env probe', () => {
  test('reports mock-mode env', () => {
    console.log(
      `[probe] NODE_ENV=${process.env.NODE_ENV} VITE_USE_MOCKS=${JSON.stringify(process.env.VITE_USE_MOCKS)} import.meta.env.VITE_USE_MOCKS=${JSON.stringify((import.meta.env as Record<string, unknown> | undefined)?.VITE_USE_MOCKS)} VITE_CANARY_AUTO=${JSON.stringify(process.env.VITE_CANARY_AUTO)} VITE_CANARY_PRELOAD=${JSON.stringify(process.env.VITE_CANARY_PRELOAD)}`,
    );
    expect(true).toBe(true);
  });
});
