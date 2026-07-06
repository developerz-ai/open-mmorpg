# ADR-0004: In-House Deterministic Physics — No Rapier / Avian

**Status:** Accepted
**Date:** 2026-07-06
**Deciders:** architecture team
**Supersedes:** —
**Related:** ADR-0001 (engine crate family and feature strategy), ADR-0003 (in-house IK and VAT)

---

## Context

The [physics spec](../../specs/game-engine/physics/README.md) names the de-facto Rust
stack — **Rapier** (`bevy_rapier`) and **Avian** — as candidates for collision, a
character controller, and scene queries, while explicitly telling us to *evaluate, not
assume*. Two constraints decide the evaluation:

1. **Determinism is a hard requirement, not a nice-to-have.** The server reuses collision
   for movement validation and anti-cheat re-simulation, and the client predicts against
   the same math ([core](../../specs/game-engine/core/README.md),
   [security](../../specs/game-server/security/README.md)). Client and server must agree
   bit-for-bit where the sim is shared. Full rigid-body solvers optimise for *fidelity*
   (contact solving, sleeping, broad hardware coverage) over *cross-machine bit
   reproducibility*, which they do not guarantee.
2. **Per ADR-0001**, the engine avoids third-party crates that may lag Bevy 0.19 and break
   the `--all-features` CI gate — the same rule that keeps `bevy_egui` and
   `bevy_kira_audio` out of the tree. A physics solver is a large surface to pin to every
   Bevy release.

What we actually need for a **cosmetic client controller** (the client predicts and
smooths; the server owns authoritative position) is small: collision shapes, shape/box
penetration, a broadphase, a kinematic move-and-slide, and ray/LOS queries. That is a poor
fit for a heavyweight dependency and an excellent fit for owned, pure code.

## Decision

Implement physics ourselves in `crates/engine-physics`, as pure, headless, deterministic
modules — no `bevy_rapier`, no `avian`, no solver.

### 1. `shapes` — collision volumes + exact penetration
`Aabb3d`, `Sphere`, `Capsule`, and a reflected `Collider` enum. `sphere_vs_aabb` is exact;
`capsule_vs_aabb` layers a closest-point refinement on top of it — the geometry a
character controller actually walks through.

### 2. `broadphase` — reuse the world quadtree
The `Broadphase` resource **reuses [`omm_world`]'s quadtree** (the same `x`/`z` index the
server uses for interest management) rather than forking a second spatial structure — one
index, no drift (CLAUDE.md). Query regions expand by the largest collider half-extent so
no overlapping box is pruned, then an exact 3D test filters candidates and the `y` axis.

### 3. `slide` — kinematic move-and-slide
Pure `move_and_slide`: horizontal slide → step-up → vertical (gravity) → floor snap, via
Gauss–Seidel depenetration of the deepest contact. Fixed iteration count and stable
contact ordering make it deterministic; it is unit-testable with no ECS.

### 4. `controller` + `query` — ECS glue and scene queries
`CharacterController` runs move-and-slide in `SimSet::Simulate`; `query` provides
ray casts against every shape and `line_of_sight` for targeting — the client-side
equivalent of the server's collision checks, both from the same open geometry.

## Consequences

### Positive
- No physics crate can lag Bevy 0.19 and break the `--all-features` gate.
- Collision and the controller are bit-deterministic and unit-tested headlessly on every
  commit — a hard requirement for shared-sim reuse a full solver could not promise.
- Small owned surface: only the shapes/queries/controller we use, no rigid-body machinery.
- The broadphase reuses the one spatial index, so it cannot drift from interest management.

### Negative / Trade-offs
- We own the controller: rigid-body dynamics, joints, and continuous collision are ours to
  add if ever needed. This batch ships kinematic move-and-slide only.
- **No continuous collision (CCD):** very fast motion can tunnel thin geometry. Per-tick
  motion is assumed smaller than collider thickness, and `CharacterController::max_fall_speed`
  caps the worst case. Documented as an honest gap in the crate.
- The character resolves against static **AABBs** (each collider's world bounding box);
  rotated/scaled collider transforms and character-vs-character contact are out of scope
  (the server is authoritative on contact between players).

### Neutral
- If a deterministic, 0.19-tracking solver later proves worth adopting for *visual-only*
  effects (ragdolls, cloth), it can layer on top without touching the shared-sim path —
  and these modules remain the determinism conformance oracle.

## Enforcement

| Rule | Enforced by |
|---|---|
| No `bevy_rapier` / `avian` in the tree | `crates/engine-physics/Cargo.toml` has no such dep; `--all-features` gate |
| Collision / controller is deterministic + guarded | `crates/engine-physics/src/slide/tests.rs`, `tests/controller_sim.rs` |
| Broadphase reuses the world quadtree | `crates/engine-physics/src/broadphase.rs` (depends on `omm-world`) |
| Physics types are reflected for MCP/agents | `lib.rs::plugin_registers_all_authored_types` |
