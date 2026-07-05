/**
 * Test preload — runs before any test module loads (wired via `bunfig.toml`).
 * Forces the web app into **mock mode** so unit/integration tests exercise the
 * real API client + Zod boundary against the in-memory backend, never a live
 * network. Deterministic by construction. → docs/specs/web-client/testing-dx
 */
process.env.VITE_USE_MOCKS ??= 'true';
