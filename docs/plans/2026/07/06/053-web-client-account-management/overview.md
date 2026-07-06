# 053 — Web Client Account Management (W5 depth)

> **Goal:** finish the account-auth spec's **"manage account"** job. PRs #50–#55 shipped register / login / logout / read-only profile + typed auth errors; this wave adds **profile & password editing**, **active-session list & revoke**, and a **reusable authed-route guard**. Server-authoritative throughout — the web submits intents, the (mock) gateway owns the truth. → [account-auth spec](../../../../specs/web-client/account-auth/README.md)

## Current State (DONE)

✅ Register / login / logout (mutations, Zod, typed `AuthError.code`)
✅ Read-only account projection (`GET /account`) + logout (`POST /auth/logout`)
✅ Typed auth-error surfacing (`authMessageKey` → `auth.error.<code>`)
✅ Registration gated by operator feature flag (`registrationOpen`)
✅ Mock gateway auth backend (`lib/mock/auth.ts`) with stable error codes
✅ 13 i18n locales, production-readiness wave (052 / PR #55) merged

## Gap (this wave)

- **No profile editing** — display name / email are read-only; spec calls for account management.
- **No password change** — a core account self-service flow is missing.
- **No active-session view** — spec lists "active sessions" as a first-class account job; a player can't see or revoke other devices.
- **No reusable route guard** — `/account` uses an inline session check; future authed surfaces have no shared guard.

## PR Batches

| PR | Scope | Files (new/edited) | Size |
|---|---|---|---|
| **053-1** | Plan + mark 052 done | `053/overview.md`, `053/status.yml`, `052/status.yml` | quick |
| **053-2** | Profile & password update | mock `PUT /account` + `POST /account/password`; `auth.ts`; `useAuth.ts`; `AccountSettings.tsx`; `Account.tsx`; catalogs ×13; tests | medium |
| **053-3** | Active sessions + revoke | mock `GET /account/sessions` + `DELETE /account/sessions/:id`; `auth.ts`; `useAuth.ts`; `ActiveSessions.tsx`; `Account.tsx`; catalogs ×13; tests | medium |
| **053-4** | Reusable route guard + e2e | `ProtectedRoute.tsx`; `App.tsx`; `Account.tsx` (drop inline check); e2e | small |

## Design (rules carried from spec + CLAUDE.md)

- **Server-authoritative.** Every change is a POST/PUT/DELETE intent to the gateway mock; the web stores only the session token. The mock simulates the gateway's stable error codes (`invalid_credentials`, `email_taken`, `wrong_password`, …) — never a leaked internal message.
- **Zod at the boundary.** Inputs (`UpdateProfileInput`, `ChangePasswordInput`, `SessionInfo`) and responses are parsed before they touch the UI.
- **TanStack Query mutations**, invalidating `['account']` (and `['account','sessions']`) on success — no hand-rolled cache.
- **SRP / ≤300 LOC.** Account route stays a thin shell composing read-only profile + `AccountSettings` + `ActiveSessions` + logout; each section is its own component.
- **i18n from day one.** Every new string via `t()` across all 13 locales; missing keys render `⟦key⟧`.
- **Dark-only semantic tokens.** No raw hex; reuse `@omm/ui` (`Card`, `Button`, `TextField`, `Alert`).

## Definition of Done (Wave)

1. A logged-in player can change their display name / email and see the projection update.
2. A logged-in player can change their password (current password required) and is logged out / re-logged on success per gateway response.
3. A logged-in player sees their active sessions and can revoke a non-current one.
4. `/account` (and any future authed route) is guarded by a reusable `ProtectedRoute` that redirects to `/login?redirect=…`.
5. All new strings present in 13 locale catalogs; `bun test` (unit + e2e) and `bin/check` green.
6. Each PR merged via `claudetm merge-pr`; CodeRabbit resolved.

## Dependencies

```
053-1 (docs) ──> 053-2 (profile+password) ──> 053-3 (sessions) ──> 053-4 (guard + e2e)
```

Sequential — each branches from main after the prior merges, so Account.tsx edits never collide.

## Links

[track 50-web-client](../../../../mvp/v1/50-web-client.md) · [account-auth spec](../../../../specs/web-client/account-auth/README.md) · [data-layer spec](../../../../specs/web-client/data-layer/README.md) · [status.yml](./status.yml) · [plan 052](../052-web-client-production-ready/overview.md)
