# 07 — AI Agent / MCP Integration (player-facing)

## Concept
Players connect **their own AI agent** to their account via MCP. Main character stays human-controlled. AI agents power a **companion system**, not autopilot.

## Design decision: companion, NOT autopilot
| Companion (chosen) | Autopilot (rejected) |
|---|---|
| agent controls a separate summoned pet/follower, bounded kit | agent drives the main character |
| adds depth, doesn't touch main-char power | = sanctioned botting |
| defensible: accessibility, economy management | breaks PvP, trivializes farming/economy |

## MCP server design
- **Per-account MCP server** (or one MCP with account-scoped auth tokens).
- Tools scoped to the **companion entity only**: `get_companion_state`, `companion_move_to`, `companion_gather`, `companion_manage_auction`, …
- **Auth** — OAuth-style token tied to account. An agent can only ever touch that player's own companion; never other players' state, never the main character directly.
- **Rate-limited** to human-plausible action rates — prevents it becoming a farming-bot vector.

## Why it's a real differentiator
No equivalent in retail WoW. Positioned as **accessibility + depth** (RSI/disability use cases, multitasking, economy management) — defensible design, not "bot-friendly WoW."

> Build these MCP tools with the same rigor as our own agent tooling → `../gold-standards-in-ai/docs/ai-agents/tools-and-mcp.md`.
