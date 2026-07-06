# ADR-0002: AAA Render Tiers — Feature Gates and Downstream Overlays

**Status:** Accepted  
**Date:** 2026-07-06  
**Deciders:** architecture team  
**Supersedes:** —  
**Related:** ADR-0001 (engine crate family and feature strategy)

---

## Context

The render pipeline has three quality tiers (Web / High / Ultra) and two techniques
that require NVIDIA hardware and non-MIT SDKs:

| Technique | SDK / dependency | License | Tier |
|---|---|---|---|
| DLSS (AI upscaling) | NVIDIA DLSS SDK | NVIDIA proprietary | Ultra |
| Solari (RT global illumination) | `bevy_solari` / NVIDIA RT libs | NVIDIA proprietary | Ultra |
| Meshlet (Nanite-style GPU cluster culling) | `bevy_pbr/meshlet` → `lz4_flex` + `range-alloc` | MIT | Ultra |

DLSS and Solari cannot appear in the workspace's `--all-features` build because
their SDKs are non-MIT and not redistributable. Meshlet is pure MIT, so it is safe
to include in `--all-features`.

We need a rule that CI enforces automatically and that a contributor reading one ADR
understands completely.

---

## Decision

### 1. DLSS and Solari are *documented downstream overlays* — never workspace features

Neither DLSS nor Solari will appear in any `[features]` table in this workspace.
They are **operator-overlay technologies**: a studio that builds on this engine and
holds the appropriate NVIDIA SDK licence may link them against a private fork or a
thin private crate; they are out of scope for the open-core release.

**Rationale:**
- MIT licence purity — shipping DLSS/Solari SDKs would change the effective licence
  of any binary that includes them.
- Hardware lock-in — DLSS and hardware-RT require NVIDIA GPUs; the engine's public
  tier ladder must work on AMD/Intel/Apple Silicon and WebGPU targets.
- Separation of concerns — the `AntiAliasing::Dlss` and `GlobalIllumination::Realtime`
  variants in `RenderTier` act as **named slots**: they document that DLSS and
  real-time GI *exist* at the Ultra tier and carry semantic meaning for agents and
  MCP tooling, but they carry zero SDK code. A downstream overlay that activates these
  slots may use any conforming implementation.

**What CI enforces:**
- `cargo build --all-features -p omm-engine-render` (and the full workspace
  `--all-features`) must compile without any DLSS/Solari SDK symbol. This is trivially
  true because neither feature flag exists.
- If a contributor adds a feature that pulls a proprietary dep, `cargo build
  --all-features` will fail with a linker error and the PR gate will not pass.

### 2. `meshlet` is an *experimental opt-in* — included in `--all-features`, headless-safe

The `meshlet` feature is declared in `omm-engine-render` and pulls only
`bevy_pbr/meshlet`, which depends on `lz4_flex` and `range-alloc` (both MIT).

`meshlet` gates device code (GPU cluster culling) but the **decision logic**
(`HeroGeometry::select`, `meshlet_compiled()`) is always compiled and headless-safe.
Under `--all-features`, CI compiles the full meshlet wiring and runs the headless
decision tests without a GPU or display. This is the pattern:

```
--no-default-features   → headless (pure logic only)
--features render       → native PBR + CSM, no meshlet
--features meshlet      → native PBR + meshlet (meshlet ⊃ render)
--all-features          → same as --features meshlet; headless tests still run
```

**Rationale:**
- MIT safety — meshlet adds no proprietary dep, so `--all-features` stays clean.
- Experimental label — meshlet virtual geometry is not shipped by default; an operator
  enables it by setting `--features meshlet` in their build profile.
- Compile-coverage — including meshlet in `--all-features` means CI always catches
  feature-gating regressions and compile errors in the meshlet path. Without it,
  meshlet could silently bitrot between PRs.
- Headless-first — `HeroGeometry::select` and `meshlet_compiled()` are pure functions
  tested headlessly even when the GPU cluster-culling code path is compiled in. This
  keeps the full test suite runnable on any CI runner.

### 3. The Ultra tier degrade ladder is the safety net

Regardless of build flags, `RenderTier::degrade_to_supported` walks
Ultra → High → Web and returns the richest tier the device's `GpuCapabilities`
can actually run. No tier is ever *silently upgraded* — a forced-Web config stays
Web even on an NVIDIA Ultra device.

The degrade ladder is the guarantee that:
- Shipping a Web/High build excludes Ultra-only techniques automatically.
- A downstream overlay can opt into DLSS/Solari by targeting the `Ultra` slot; the
  fallback to `High` or `Web` is automatic on non-NVIDIA hardware.

---

## Consequences

### Positive
- `--all-features` is always safe to run in CI on any platform; no proprietary dep
  can leak in through a feature flag.
- DLSS/Solari are explicitly documented as named slots in the tier enum — not missing
  by accident but excluded by design. Downstream studios have a clear extension point.
- Meshlet compile-coverage in CI catches wiring regressions before they reach clients.
- The tier degrade ladder provides a hardware-agnostic safety net for all tiers.

### Negative / Trade-offs
- Operators who want DLSS/Solari must maintain a private overlay crate and manage
  the NVIDIA SDK dependency themselves — the open-core engine gives them the *slot*
  but not the *wire*.
- `AntiAliasing::Dlss` and `GlobalIllumination::Realtime` variants compile but do
  nothing without a downstream overlay. This is intentional but may surprise
  contributors who expect them to light up immediately on an NVIDIA GPU.

### Neutral
- Meshlet remains behind a feature flag — default builds use discrete LOD + imposters.
  Ultra tier + meshlet build → meshlet. Any other combination → discrete LOD.

---

## Enforcement

| Rule | Enforced by |
|---|---|
| No DLSS/Solari feature in workspace | `cargo build --all-features` gate — would fail at link time |
| `--all-features` compiles headless | `crates/engine-render/tests/tier_fallback.rs` (see below) |
| Meshlet decision is headless-correct | `tier_fallback.rs` + `aaa.rs` integration tests |
| Degrade ladder is total and monotone | `tier_fallback.rs` + `tier.rs` integration tests |
| DLSS/Solari slots have correct semantics | `tier.rs::anti_aliasing_matches_tier_spec` + `global_illumination_matches_tier_spec` |
