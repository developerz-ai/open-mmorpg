# 07 — MCP Companion Agents (player-facing)

> Players connect their own AI agent to control a **companion** entity — never the main character. Design + rationale: [../initial-idea/07-ai-agent-mcp-integration.md](../initial-idea/07-ai-agent-mcp-integration.md).

## Where it lives
`apps/mcp` — a per-account (or account-scoped-token) MCP server. It is **just another authenticated client** of the shard/worldsvc — it has no privileged path.

## Tool surface (companion-scoped only)
```
get_companion_state      companion_move_to      companion_gather
companion_attack         companion_manage_auction    companion_follow
```
- Every tool call resolves to the caller's **own companion entity** and nothing else.
- Companion has a **bounded kit** (limited power budget) — an assist, not a replacement.

## Guardrails (non-negotiable)
| Guard | Rule |
|---|---|
| Scope | token → account → that account's companion only. Cannot read/act on other players or the main char. |
| Authority | goes through the **same server-authoritative path** as a human client. No shortcut into sim/DB. |
| Rate limit | clamped to **human-plausible** action rates — kills the farm-bot vector. |
| Economy | `companion_manage_auction` etc. mutate ownership only via `persistence` transactions ([04](04-data-and-consistency.md)). |
| Audit | every companion action is logged with account + token id. |

## Build standard
Design these MCP tools to the same bar as our internal agent tooling — one client per provider, typed errors, narrow capabilities → `../../gold-standards-in-ai/docs/ai-agents/tools-and-mcp.md`.
