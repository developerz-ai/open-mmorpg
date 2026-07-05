# Input

> Device events → **logical actions** → [`Intent`](../../../../crates/protocol/src/lib.rs). The client sends *actions*, never raw keys or state — which is exactly the server-authoritative contract, and exactly what an AI agent injects instead of a keyboard. → [netcode](../../game-server/netcode/README.md)

## What it does
Maps keyboard/mouse/gamepad input to an enum of **logical actions** (Move, CastAbility, Interact…), which the client turns into stamped [`Intent`](../../../../crates/protocol/src/lib.rs)s. The Rust standard is [`leafwing-input-manager`](https://github.com/Leafwing-Studios/leafwing-input-manager): derive `Actionlike`, attach an `InputMap`, read a per-entity `ActionState` — many-to-many device→action binding, deadzones, and **rebindable keybinds** built in.

## Design
- **Intent, not device events.** Game logic reads *actions*, never scancodes. This is the natural fit for a [server-authoritative](../../game-server/security/README.md) model (client sends intent) — and it means an **agent** feeds the same action stream a human's keyboard produces, no separate code path ([ai-native-dx](../../game-engine/ai-native-dx/README.md), [prediction-core](../prediction-core/README.md)).
- **Rebindable + gamepad from day one.** `InputMap` is data — keybinds are user-editable config, not hardcoded; gamepad and keyboard map to the same actions. Axis inputs (analog stick, look) get deadzone/sensitivity processing.
- **Local echo through prediction.** An action becomes an `Intent`, applied immediately to the [prediction core](../prediction-core/README.md) *and* sent to the server — zero input latency locally, authoritative correction later ([networking](../networking/README.md)).
- **Headless = inject actions directly.** With no window, tests and agents push `ActionState`/`Intent` straight in — the input layer is bypassable because actions, not devices, are the interface ([platform](../platform/README.md)).

## Distilled from the references
| Source | Adopt |
|---|---|
| `leafwing-input-manager` | Action enum + `InputMap` + `ActionState`; rebindable, gamepad, axis processing |
| Server-authoritative model | Client sends **actions/intents**, never device state ([security](../../game-server/security/README.md)) |
| [ai-native-dx](../../game-engine/ai-native-dx/README.md) | Actions as the interface → agents inject the same stream as humans |

## Rules
- **Actions, not raw keys**, are the game's input interface — devices bind to actions, logic reads actions.
- Every action becomes a stamped **`Intent`**; the client never sends state ([netcode](../../game-server/netcode/README.md)).
- Keybinds are **user-editable data**; gamepad + keyboard share the action set from day one.
- Headless path injects actions directly — no device layer required ([platform](../platform/README.md)).

## Links
[prediction-core](../prediction-core/README.md) · [networking](../networking/README.md) · [hud-ui](../hud-ui/README.md) · [platform](../platform/README.md) · [netcode](../../game-server/netcode/README.md) · [ai-native-dx](../../game-engine/ai-native-dx/README.md)
