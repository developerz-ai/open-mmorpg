# Account & Auth

> Registration, login, sessions, account settings. **Server-authoritative:** [`apps/gateway`](../../../../apps/gateway) owns sessions, tokens, passwords, 2FA. The web submits credentials and **intents** — it stores only a session token, validates every response with **Zod**, and never sees a password hash or internal error. The brittle PHP account panel done right.

## What it does
The account surface of the operator web ([SPA](../app-shell/README.md), logged-in). Four jobs: **register**, **log in**, **manage session**, **manage account** (email, password change, 2FA enrolment, active sessions). Every one is a POST of an intent to the gateway; the gateway is the sole authority and the web renders its typed response.

## The authority split
| Concern | Owner | Web's role |
|---|---|---|
| Password hashing, verification, reset | **gateway** | submits plaintext over TLS to a `t()`'d form; never stores, never hashes |
| Session token / refresh | **gateway** | stores the opaque token (httpOnly cookie preferred), attaches it, never mints one |
| 2FA (TOTP/enrolment) | **gateway** | renders the enrolment challenge + collects the code |
| Registration open/closed | **operator feature flag** ([operator-brand](../operator-brand/README.md)) | hides/shows the form; server re-enforces |
| Rate-limit, lockout, CAPTCHA gate | **gateway** ([security](../../game-server/security/README.md)) | surfaces the typed refusal, never decides it |

## Design
- **Zod at the boundary.** Every gateway response (`Session`, `Account`, `AuthError`) parsed before it reaches the UI — mirrors [`realm.ts`](../../../../apps/web/src/lib/realm.ts). Bad shape → typed error, not a render.
- **TanStack Query** for account reads; **mutations** for login/register/settings, invalidating account state on success.
- **Typed client error codes** only — `invalid_credentials`, `registration_closed`, `rate_limited`, `twofa_required`. Never surface a stack, SQL, or internal message; the gateway maps internals to stable codes.
- **Session token is the only secret the web holds.** No account truth cached beyond the current session projection. Logout revokes server-side.
- **i18n from day one** — every label, error, and email string via `t()`; missing keys render `⟦key⟧`.

## Distilled from
| Source | Lesson | Our verdict |
|---|---|---|
| Private-server PHP account panels (plaintext passwords, SQL-injection, leaked errors) | Client-side auth logic + string-concatenated SQL + raw DB errors = the classic account-breach vector | **Fix** — auth is gateway-authoritative; web submits intents, holds only a token, shows typed codes |
| Silent/ambiguous "login failed" screens | Undifferentiated failures hide lockout, 2FA, closed-registration states from the user | **Keep** the UX, **fix** the contract — stable typed codes, each with a `t()`'d message |

## Rules
- **The web never owns auth.** Sessions, tokens, passwords, 2FA live in the gateway; the web submits and renders.
- **Store only the session token.** No password, no hash, no account secret in web state.
- **Zod-validate every auth response** before it touches the UI.
- **Typed client error codes** — never leak internal errors, SQL, or credentials to UI or logs.
- **Registration open/closed is an operator flag** — hide the form, but the server is the enforcement point.
- **Every string via `t()`** — forms, errors, transactional email copy.

## Links
[app-shell](../app-shell/README.md) · [data-layer](../data-layer/README.md) · [operator-brand](../operator-brand/README.md) · [i18n](../i18n/README.md) · [architecture/09](../../../architecture/09-operator-web.md) · [game-server](../../game-server/README.md) · [security](../../game-server/security/README.md) · [`apps/gateway`](../../../../apps/gateway) · [`realm.ts`](../../../../apps/web/src/lib/realm.ts) · [CLAUDE.md](../../../../CLAUDE.md)
