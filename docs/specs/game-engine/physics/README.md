# Physics & Collision

> Rust-native collision, a **kinematic character controller**, and spatial/line-of-sight queries. On the client physics is **cosmetic** — the server owns truth — but the same deterministic collision math is available server-side for movement validation. → [engine README](../README.md)

## What it does
Provides collision shapes, scene queries (ray/shape casts), and a character controller. The de-facto Rust stack is **Rapier** (`bevy_rapier`); the ECS-idiomatic alternative is **Avian** (pure-Bevy). Rapier is mature and Rust-native — the one area with no meaningful Bevy-vs-Unreal gap for our use case. → [Rapier character controller](https://rapier.rs/docs/user_guides/bevy_plugin/character_controller/), [scene queries](https://rapier.rs/docs/user_guides/bevy_plugin/scene_queries/).

## Design
- **Kinematic character controller.** `KinematicCharacterController` slides bodies along obstacles with auto step-climb, floor snapping, and moving-platform support — right for a client that **predicts and visualizes** movement while the server owns the authoritative position ([prediction-core](../../client/prediction-core/README.md)).
- **Scene queries for gameplay.** Ray/shape casts for **line-of-sight**, targeting, and interaction probes — the client-side equivalent of the server's collision checks derived from the same open glTF geometry ([world-model VMaps role](../../game-server/world-model/README.md)).
- **Deterministic where shared.** Collision used by the server for [movement validation](../../game-server/security/README.md) or re-sim must be deterministic (fixed timestep, stable ordering) so client prediction and server authority agree ([core](../core/README.md)). Purely visual client physics (ragdolls, cloth) need not be.
- **Client physics never asserts state.** The server validates movement against its own collision; the client uses physics to *predict* and *smooth*, not to claim position ([security](../../game-server/security/README.md)).

## Distilled from the references
| Source | Adopt |
|---|---|
| Rapier (`bevy_rapier`) | Character controller + ray/shape-cast scene queries; mature Rust-native default |
| Avian (ECS physics) | Alternative if ECS-idiomatic integration beats Rapier's wrapper — evaluate, don't assume |
| Server collision from open geometry | LOS/height/cover from our glTF/heightmap meshes, not extracted client data ([world-model](../../game-server/world-model/README.md)) |

## Rules
- **Server is authoritative on position.** Client physics predicts and smooths; it never sends state ([security](../../game-server/security/README.md), [prediction-core](../../client/prediction-core/README.md)).
- **Deterministic collision where the server reuses it** — fixed timestep, stable contact ordering ([core](../core/README.md)).
- Collision geometry derives from our **open** glTF/heightmap source — never an extracted client mesh ([legal](../../../initial-idea/01-legal-and-licensing.md)).

## Links
[core](../core/README.md) · [animation](../animation/README.md) · [world-model](../../game-server/world-model/README.md) · [security](../../game-server/security/README.md) · [prediction-core](../../client/prediction-core/README.md)
