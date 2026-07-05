# Security Policy

Anti-cheat and anti-dupe are the **#1 priority** of this engine — it's what historically kills private servers. We take security reports seriously.

## Supported versions

Pre-1.0: only the latest `main` is supported. Please base reports on the current `main`.

| Version | Supported |
|---|---|
| `main` (latest) | ✅ |
| anything older | ❌ |

## Reporting a vulnerability

**Please do not open a public issue for a security vulnerability.**

Report privately, ideally via **GitHub Security Advisories** — use the **"Report a vulnerability"** tab under this repository's **Security** section. That opens a private channel with the maintainers.

If you can't use GitHub advisories, email **[security@developerz.ai](mailto:security@developerz.ai)**.

Please include enough to reproduce: affected component, steps, impact, and any proof-of-concept. We'll acknowledge receipt, work with you on a fix, and coordinate disclosure. We ask for a reasonable disclosure window — up to **90 days** — before any public write-up, and we'll credit you unless you prefer otherwise.

## Scope

In scope — anything that breaks server authority or the integrity of the world:

- **Anti-dupe:** any path that duplicates items, currency, or ownership, or that mutates ownership outside a YugabyteDB transaction.
- **Anti-cheat:** client asserting state instead of intent; bypassing server-side validation (range, cooldown, cost, line-of-sight, movement speed/teleport).
- **Sandbox escape:** operator WASM/Lua reaching beyond its capability API (DB, cache, filesystem, network) or evading fuel metering.
- **Auth / session:** session hijack, token forgery, privilege escalation at the gateway or shard.
- **Data exposure:** leaking credentials or internal detail through errors, logs, or the wire protocol.

Out of scope: issues that require a malicious operator on their own realm's own data, best-practice suggestions with no exploit, and third-party infrastructure you don't control.

For the full model, see [docs/architecture/08-security-anticheat.md](docs/architecture/08-security-anticheat.md) and the anti-dupe contract in [docs/architecture/04-data-and-consistency.md](docs/architecture/04-data-and-consistency.md).
