# ADR-0003: In-House IK and VAT Math — No Ecosystem Animation Crates

**Status:** Accepted
**Date:** 2026-07-06
**Deciders:** architecture team
**Supersedes:** —
**Related:** ADR-0001 (engine crate family and feature strategy)

---

## Context

Bevy core ships **no inverse kinematics** and **no vertex-animation-texture (VAT)**
support. The [animation spec](../../specs/game-engine/animation/README.md) originally
named three community crates to fill the gap:

| Need | Crate named in the spec | Concern |
|---|---|---|
| Two-bone / FSM / blend-space graph | `bevy_animation_graph` | Tracks Bevy closely but is a large surface we do not need most of. |
| FABRIK chain IK | `bevy_mod_inverse_kinematics` | Small, but ecosystem crates lag major Bevy releases. |
| Crowd VAT | (no crate) | No maintained option; the technique is a handful of pure math. |

Per **ADR-0001**, the engine avoids third-party crates that may lag Bevy 0.19 and
break the `--all-features` CI gate (the same rule that keeps `bevy_rapier`/`avian`,
`bevy_egui`, and `bevy_kira_audio` out of the tree). The animation-critical math we
actually need — a limb IK solver, a general chain solver, and VAT frame/index
addressing — is small, pure, and must be **bit-deterministic where shared** with the
server sim. That is a poor fit for a dependency and an excellent fit for owned code.

## Decision

Implement the pieces ourselves in `crates/engine-anim`, as pure headless modules:

### 1. `ik` — two solvers, both deterministic

- **`solve_two_bone`** — an exact, single-pass, law-of-cosines limb solver for a
  three-joint chain (shoulder→elbow→wrist). A **pole** hint fixes the bend plane so
  the elbow/knee never flips. This is the correct tool for arms/legs and replaces
  the `bevy_animation_graph` two-bone node.
- **`IkChain::solve`** — iterative **FABRIK** over an N-joint chain with fixed
  segment lengths, bounded iterations, and unreachable-target straightening. Replaces
  `bevy_mod_inverse_kinematics`.

Both are pure `f32` math with fixed iteration order/count, degenerate-input guards
(no `NaN`), and fail-loud validation — so the server and every client produce
identical output for anything reused in the shared sim.

### 2. `vat` — pure frame + index math

`VatClip::sample_at` (loop/once/ping-pong frame selection + blend factor) and
`VatLayout` (row-major texel dimensions, flat bake index, sample UVs). The offline
baker, the device shader, and headless CI all agree on these indices. GPU baking and
shader playback remain the client's job and are out of scope for headless CI.

## Consequences

### Positive
- No animation crate can lag Bevy 0.19 and break the `--all-features` gate.
- IK and VAT math are bit-deterministic and unit-tested headlessly on every commit —
  a hard requirement for shared-sim reuse that a third-party crate could not promise.
- Small owned surface: we ship only the solvers we use, no unused graph machinery.

### Negative / Trade-offs
- We own the solvers: FSM/blend-space authoring, if needed later, is ours to build
  (this batch ships blend primitives + two IK solvers + VAT math, not a visual graph
  editor or state machine — see the crate's honest-gaps note).
- GPU-side VAT bake and skinning still require a device and are verified on the client
  track, not in headless CI.

### Neutral
- If a mature, 0.19-tracking IK/graph crate later proves worth adopting, these modules
  are a drop-in-compatible reference and a determinism conformance oracle.

## Enforcement

| Rule | Enforced by |
|---|---|
| No ecosystem IK/graph crate in the tree | `crates/engine-anim/Cargo.toml` has no such dep; `--all-features` gate |
| IK is deterministic + guarded | `crates/engine-anim/tests/ik.rs` |
| VAT frame/index math is correct + fail-loud | `crates/engine-anim/tests/vat.rs` |
| Solver/VAT types are reflected for MCP/agents | `lib.rs::anim_types_are_registered` |
