//! **AAA hero-asset rendering paths** — meshlet virtual geometry for the highest
//! detail "hero" assets (player characters, boss models, key set-pieces), with a
//! deterministic degrade to the discrete-LOD baseline that runs everywhere.
//!
//! Virtual geometry ([Nanite]-style GPU cluster culling) is **opt-in per asset**,
//! never the default path ([spec]): it needs both the native [`RenderTier::Ultra`]
//! capabilities *and* a build compiled with the `meshlet` feature. Anything less —
//! [`High`](RenderTier::High)/[`Web`](RenderTier::Web), or a build without the
//! feature — falls back to the discrete-LOD chains + imposters every tier already
//! ships.
//!
//! # Headless-first
//! The path *decision* ([`HeroGeometry::select`]) is pure, total logic — a headless
//! agent reasons about which geometry a hero asset resolves to exactly as the
//! rendered client does at boot. The meshlet device wiring ([`add_meshlet_hero_path`])
//! is gated behind the `meshlet` feature and only compiled for native AAA heads.
//!
//! # Licensing
//! `meshlet` is MIT — `bevy_pbr/meshlet` pulls only `lz4_flex` + `range-alloc` — so
//! it is a first-class workspace feature, safe under `--all-features`. The sibling
//! NVIDIA-only paths, DLSS (`bevy_render/dlss`) and Solari (`bevy_pbr/solari`), are
//! **never** exposed as workspace features: non-MIT, and they would contaminate the
//! open-core license and `--all-features` ([ADR-0001]). A downstream commercial
//! overlay may enable them; the open core does not.
//!
//! [Nanite]: <https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine>
//! [spec]: <../../../docs/specs/game-engine/rendering/README.md>
//! [ADR-0001]: <../../../docs/architecture/decisions/0001-engine-crate-family-and-features.md>

use bevy_ecs::prelude::{Component, ReflectComponent};
use bevy_reflect::{std_traits::ReflectDefault, Reflect};

use crate::tier::RenderTier;

/// Marks an entity as a **hero asset**: high-poly, high-priority geometry that
/// takes the richest path the device + build can offer (meshlet virtual geometry on
/// Ultra, discrete LOD everywhere else). A plain marker — the concrete geometry
/// representation is chosen by [`HeroGeometry::select`], keeping the content-side
/// tag free of any render-tier knowledge so the same scene data drives every tier.
#[derive(Component, Reflect, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[reflect(Component, Debug, Default, PartialEq)]
pub struct HeroAsset;

/// The geometry representation resolved for a hero asset. One tag drives the asset
/// pipeline: load the pre-processed meshlet mesh, or the discrete-LOD chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum HeroGeometry {
    /// Meshlet virtual geometry — GPU cluster culling + per-pixel LOD via
    /// [`add_meshlet_hero_path`]. Requires [`RenderTier::Ultra`] **and** a
    /// `meshlet`-feature build.
    Meshlet,
    /// Discrete LOD chain + imposters — the baseline that runs on every tier and
    /// every build. The graceful degrade target when meshlet is unavailable.
    DiscreteLod,
}

impl HeroGeometry {
    /// Select the geometry path for a hero asset from the resolved render `tier`
    /// and whether this build compiled the meshlet backend (`meshlet_available`).
    ///
    /// Meshlet needs **both**: the Ultra tier's native capabilities *and* the
    /// compiled backend. Missing either degrades to [`DiscreteLod`](Self::DiscreteLod)
    /// — per the spec the path is opt-in, never a hard requirement. Pure, total and
    /// deterministic: same inputs → same path, in any build.
    #[must_use]
    pub fn select(tier: RenderTier, meshlet_available: bool) -> Self {
        if meshlet_available && tier.virtual_geometry() {
            HeroGeometry::Meshlet
        } else {
            HeroGeometry::DiscreteLod
        }
    }

    /// [`select`](Self::select) using this build's compiled meshlet capability
    /// ([`meshlet_compiled`]) — the convenience the client calls at asset-load time.
    #[must_use]
    pub fn for_current_build(tier: RenderTier) -> Self {
        Self::select(tier, meshlet_compiled())
    }

    /// Whether this path is meshlet virtual geometry (vs the discrete-LOD baseline).
    #[must_use]
    pub fn is_meshlet(self) -> bool {
        matches!(self, HeroGeometry::Meshlet)
    }
}

/// Whether this build compiled the meshlet virtual-geometry backend (built with
/// `--features meshlet`). `const` so path logic folds it at compile time; a headless
/// build without the feature always answers `false`, so [`HeroGeometry::select`]
/// there can only ever yield the discrete-LOD baseline.
#[must_use]
pub const fn meshlet_compiled() -> bool {
    cfg!(feature = "meshlet")
}

/// The meshlet asset (`MeshletMesh`) and its render component (`MeshletMesh3d`),
/// re-exported so hero-asset content can attach meshlet geometry to a
/// [`HeroAsset`] entity. Present only under the `meshlet` feature.
#[cfg(feature = "meshlet")]
pub use bevy_pbr::experimental::meshlet::{MeshletMesh, MeshletMesh3d};
#[cfg(feature = "meshlet")]
pub use meshlet_path::{add_meshlet_hero_path, HERO_CLUSTER_BUFFER_SLOTS};

/// Meshlet device wiring — compiled only for native AAA heads (`--features
/// meshlet`, a strict superset of `render`). Isolated in a submodule so the pure
/// path logic above stays free of any GPU dependency.
#[cfg(feature = "meshlet")]
mod meshlet_path {
    use bevy_app::App;
    use bevy_pbr::experimental::meshlet::MeshletPlugin;

    /// Pre-allocated meshlet cluster slots (4 bytes VRAM each → 256 KiB). Sized for
    /// a handful of on-screen hero assets, not a Nanite-dense city — raise it if
    /// hero geometry blinks or drops out. Must stay ≤ 2^25 (Bevy's hard cap).
    pub const HERO_CLUSTER_BUFFER_SLOTS: u32 = 65_536;

    /// Add the meshlet virtual-geometry backend so [`HeroAsset`](super::HeroAsset)
    /// entities carrying a [`MeshletMesh3d`](super::MeshletMesh3d) render via GPU
    /// cluster culling. Idempotent; call **after** the core render plugins — it
    /// needs the render sub-app they create.
    pub fn add_meshlet_hero_path(app: &mut App) {
        if !app.is_plugin_added::<MeshletPlugin>() {
            app.add_plugins(MeshletPlugin {
                cluster_buffer_slots: HERO_CLUSTER_BUFFER_SLOTS,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meshlet_only_on_ultra_with_a_meshlet_build() {
        // The one corner that lights up the meshlet path.
        assert_eq!(
            HeroGeometry::select(RenderTier::Ultra, true),
            HeroGeometry::Meshlet
        );
        // Ultra hardware but a build without the backend → baseline, not meshlet.
        assert_eq!(
            HeroGeometry::select(RenderTier::Ultra, false),
            HeroGeometry::DiscreteLod
        );
    }

    #[test]
    fn non_ultra_tiers_never_use_meshlet() {
        // Meshlet is Ultra-only; High/Web always degrade regardless of the build.
        for tier in [RenderTier::High, RenderTier::Web] {
            for available in [true, false] {
                assert_eq!(
                    HeroGeometry::select(tier, available),
                    HeroGeometry::DiscreteLod,
                    "{tier:?} must degrade to discrete LOD (meshlet is Ultra-only)"
                );
            }
        }
    }

    #[test]
    fn meshlet_requires_both_tier_and_build() {
        // The AND gate, spelled out: only (Ultra, available) is meshlet.
        assert!(HeroGeometry::select(RenderTier::Ultra, true).is_meshlet());
        assert!(!HeroGeometry::select(RenderTier::Ultra, false).is_meshlet());
        assert!(!HeroGeometry::select(RenderTier::High, true).is_meshlet());
        assert!(!HeroGeometry::select(RenderTier::Web, true).is_meshlet());
    }

    #[test]
    fn for_current_build_tracks_the_compiled_feature() {
        // Ultra resolves by whether *this* build compiled the backend.
        let expected = if meshlet_compiled() {
            HeroGeometry::Meshlet
        } else {
            HeroGeometry::DiscreteLod
        };
        assert_eq!(HeroGeometry::for_current_build(RenderTier::Ultra), expected);
        // Non-Ultra stays baseline no matter the build.
        assert_eq!(
            HeroGeometry::for_current_build(RenderTier::Web),
            HeroGeometry::DiscreteLod
        );
    }

    #[test]
    fn meshlet_compiled_matches_cfg() {
        assert_eq!(meshlet_compiled(), cfg!(feature = "meshlet"));
    }
}
