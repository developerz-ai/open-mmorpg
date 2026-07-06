//! Capability-driven **render tiers** — pick a feature level from the GPU
//! capabilities queried at boot, with no per-feature code forks.
//!
//! One material set, three tiers ([spec]): [`RenderTier::Ultra`] (native — meshlet
//! virtual geometry + Solari real-time GI + DLSS), [`RenderTier::High`] (native —
//! discrete LOD + baked GI + TAA), [`RenderTier::Web`] (the WebGPU baseline —
//! forward + SMAA). Selection is pure, total and deterministic: [`GpuCapabilities`]
//! in, a tier out — so a headless test reasons about the tier ladder exactly as the
//! rendered client does at boot.
//!
//! An AAA technique that only works on one vendor is **never the default** — it is
//! a tier gated behind a capability query. The web target assumes only the WebGPU
//! baseline: no bindless, no ray tracing, no mesh shaders.
//!
//! [spec]: <../../../docs/specs/game-engine/rendering/README.md>

use bevy_reflect::{std_traits::ReflectDefault, Reflect};

use crate::error::RenderError;

/// GPU capabilities queried once at boot. Every field is a hard prerequisite for
/// some AAA path; tier selection reads them and never re-queries per frame.
///
/// [`Default`] is the **safe floor** — the WebGPU baseline (no native backend, no
/// advanced features) — so an un-probed device can only ever land on
/// [`RenderTier::Web`], never over-claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub struct GpuCapabilities {
    /// A native desktop backend (Vulkan/Metal/DX12), as opposed to the browser's
    /// WebGPU. The AAA tiers are native-only; the web target is the baseline.
    pub native_backend: bool,
    /// Compute shaders — required for meshlet cluster culling and GPU-driven draw
    /// submission.
    pub compute: bool,
    /// Bindless resource arrays (an experimental native-only wgpu extension) —
    /// required for GPU-driven, bindless draw submission.
    pub bindless: bool,
    /// Indirect multi-draw — the CPU records one call, the GPU expands it, so draw
    /// cost stays flat as the scene grows.
    pub multi_draw_indirect: bool,
    /// Hardware ray tracing — required for Solari (ReSTIR) real-time GI. NVIDIA-only
    /// today, hence an [`RenderTier::Ultra`]-only opt-in.
    pub ray_tracing: bool,
}

impl GpuCapabilities {
    /// The WebGPU baseline: a browser/fallback device with none of the native
    /// extensions. Resolves to [`RenderTier::Web`].
    #[must_use]
    pub const fn web_baseline() -> Self {
        Self {
            native_backend: false,
            compute: false,
            bindless: false,
            multi_draw_indirect: false,
            ray_tracing: false,
        }
    }

    /// A native desktop GPU without hardware ray tracing: GPU-driven rendering but
    /// no real-time GI. Resolves to [`RenderTier::High`].
    #[must_use]
    pub const fn native_high() -> Self {
        Self {
            native_backend: true,
            compute: true,
            bindless: true,
            multi_draw_indirect: true,
            ray_tracing: false,
        }
    }

    /// A native RTX-class GPU with every AAA prerequisite. Resolves to
    /// [`RenderTier::Ultra`].
    #[must_use]
    pub const fn native_ultra() -> Self {
        Self {
            ray_tracing: true,
            ..Self::native_high()
        }
    }
}

/// Which temporal/spatial anti-aliasing a tier runs. There is no cross-vendor
/// temporal upscaler, so only the native RTX path gets [`AntiAliasing::Dlss`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum AntiAliasing {
    /// DLSS temporal upscaling — native NVIDIA only ([`RenderTier::Ultra`]).
    Dlss,
    /// Temporal anti-aliasing — native baseline ([`RenderTier::High`]).
    Taa,
    /// Subpixel morphological AA — the spatial, cross-vendor fallback
    /// ([`RenderTier::Web`]).
    Smaa,
}

/// How a tier resolves indirect lighting. Baked-first is the cross-platform
/// default; real-time is the RTX opt-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum GlobalIllumination {
    /// Baked irradiance volumes + light probes — the default everywhere.
    Baked,
    /// Solari (ReSTIR DI+GI) real-time GI — hardware-ray-traced, RTX-only opt-in
    /// ([`RenderTier::Ultra`]).
    Realtime,
}

/// A render feature level. One material set drives all three; the tier only gates
/// which techniques run. Ordered richest → leanest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum RenderTier {
    /// Native AAA: meshlet virtual geometry + Solari real-time GI + DLSS.
    Ultra,
    /// Native baseline: discrete LOD/imposters + baked GI + TAA.
    High,
    /// WebGPU baseline: clustered-forward + SMAA. No bindless/RT/mesh shaders.
    Web,
}

impl RenderTier {
    /// Select the richest tier the device supports. Total and deterministic — an
    /// un-probed (default) [`GpuCapabilities`] yields [`RenderTier::Web`], never a
    /// tier the hardware can't run.
    #[must_use]
    pub fn select(caps: &GpuCapabilities) -> Self {
        if caps.native_backend
            && caps.compute
            && caps.bindless
            && caps.multi_draw_indirect
            && caps.ray_tracing
        {
            RenderTier::Ultra
        } else if caps.native_backend {
            RenderTier::High
        } else {
            RenderTier::Web
        }
    }

    /// Fail loud if `caps` cannot run this tier, naming the first missing
    /// capability. Use when a config *demands* a specific tier (e.g. forcing Ultra)
    /// rather than accepting [`select`](Self::select)'s best-supported answer.
    ///
    /// # Errors
    /// [`RenderError::MissingCapability`] naming the first unmet prerequisite.
    pub fn ensure_supported_by(self, caps: &GpuCapabilities) -> Result<(), RenderError> {
        let missing = match self {
            // The baseline runs anywhere a WebGPU device exists.
            RenderTier::Web => None,
            RenderTier::High => (!caps.native_backend).then_some("native_backend"),
            RenderTier::Ultra => {
                if !caps.native_backend {
                    Some("native_backend")
                } else if !caps.compute {
                    Some("compute")
                } else if !caps.bindless {
                    Some("bindless")
                } else if !caps.multi_draw_indirect {
                    Some("multi_draw_indirect")
                } else if !caps.ray_tracing {
                    Some("ray_tracing")
                } else {
                    None
                }
            }
        };
        match missing {
            Some(capability) => Err(RenderError::MissingCapability {
                capability: capability.to_owned(),
            }),
            None => Ok(()),
        }
    }

    /// The anti-aliasing / upscaling this tier runs.
    #[must_use]
    pub fn anti_aliasing(self) -> AntiAliasing {
        match self {
            RenderTier::Ultra => AntiAliasing::Dlss,
            RenderTier::High => AntiAliasing::Taa,
            RenderTier::Web => AntiAliasing::Smaa,
        }
    }

    /// How this tier resolves global illumination.
    #[must_use]
    pub fn global_illumination(self) -> GlobalIllumination {
        match self {
            RenderTier::Ultra => GlobalIllumination::Realtime,
            RenderTier::High | RenderTier::Web => GlobalIllumination::Baked,
        }
    }

    /// Whether meshlet virtual geometry (Nanite-style GPU cluster culling) runs.
    /// Ultra only; other tiers use discrete LOD chains + imposters.
    #[must_use]
    pub fn virtual_geometry(self) -> bool {
        matches!(self, RenderTier::Ultra)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_capabilities_are_the_web_baseline() {
        // An un-probed device must never over-claim: default = the safe floor.
        assert_eq!(GpuCapabilities::default(), GpuCapabilities::web_baseline());
        assert_eq!(
            RenderTier::select(&GpuCapabilities::default()),
            RenderTier::Web
        );
    }

    #[test]
    fn full_capability_set_selects_ultra() {
        assert_eq!(
            RenderTier::select(&GpuCapabilities::native_ultra()),
            RenderTier::Ultra
        );
    }

    #[test]
    fn native_without_ray_tracing_selects_high() {
        assert_eq!(
            RenderTier::select(&GpuCapabilities::native_high()),
            RenderTier::High
        );
    }

    #[test]
    fn ray_tracing_without_native_backend_stays_web() {
        // Vendor features are meaningless without a native backend — never fork
        // onto a web device that merely reports a flag.
        let caps = GpuCapabilities {
            ray_tracing: true,
            compute: true,
            bindless: true,
            multi_draw_indirect: true,
            native_backend: false,
        };
        assert_eq!(RenderTier::select(&caps), RenderTier::Web);
    }

    #[test]
    fn native_missing_one_ultra_prerequisite_falls_to_high() {
        // Each AAA prerequisite is load-bearing: drop any one and Ultra is out.
        let tweaks: [fn(&mut GpuCapabilities); 4] = [
            |c| c.compute = false,
            |c| c.bindless = false,
            |c| c.multi_draw_indirect = false,
            |c| c.ray_tracing = false,
        ];
        for tweak in tweaks {
            let mut caps = GpuCapabilities::native_ultra();
            tweak(&mut caps);
            assert_eq!(RenderTier::select(&caps), RenderTier::High);
        }
    }

    #[test]
    fn web_tier_is_supported_by_every_device() {
        assert!(RenderTier::Web
            .ensure_supported_by(&GpuCapabilities::web_baseline())
            .is_ok());
    }

    #[test]
    fn ensure_high_names_native_backend_gap() {
        let err = RenderTier::High
            .ensure_supported_by(&GpuCapabilities::web_baseline())
            .expect_err("web baseline cannot run the native High tier");
        assert_eq!(
            err,
            RenderError::MissingCapability {
                capability: "native_backend".to_owned()
            }
        );
    }

    #[test]
    fn ensure_ultra_reports_first_missing_capability_in_order() {
        // Web baseline: the native backend is the first gap reported.
        let err = RenderTier::Ultra
            .ensure_supported_by(&GpuCapabilities::web_baseline())
            .expect_err("web baseline cannot run Ultra");
        assert_eq!(
            err,
            RenderError::MissingCapability {
                capability: "native_backend".to_owned()
            }
        );
        // Native but no RT: ray tracing is the reported gap.
        let err = RenderTier::Ultra
            .ensure_supported_by(&GpuCapabilities::native_high())
            .expect_err("native High device cannot run Ultra");
        assert_eq!(
            err,
            RenderError::MissingCapability {
                capability: "ray_tracing".to_owned()
            }
        );
        // Fully capable: Ok.
        assert!(RenderTier::Ultra
            .ensure_supported_by(&GpuCapabilities::native_ultra())
            .is_ok());
    }

    #[test]
    fn tier_feature_matrix_matches_spec() {
        assert_eq!(RenderTier::Ultra.anti_aliasing(), AntiAliasing::Dlss);
        assert_eq!(RenderTier::High.anti_aliasing(), AntiAliasing::Taa);
        assert_eq!(RenderTier::Web.anti_aliasing(), AntiAliasing::Smaa);

        assert_eq!(
            RenderTier::Ultra.global_illumination(),
            GlobalIllumination::Realtime
        );
        assert_eq!(
            RenderTier::High.global_illumination(),
            GlobalIllumination::Baked
        );
        assert_eq!(
            RenderTier::Web.global_illumination(),
            GlobalIllumination::Baked
        );

        assert!(RenderTier::Ultra.virtual_geometry());
        assert!(!RenderTier::High.virtual_geometry());
        assert!(!RenderTier::Web.virtual_geometry());
    }

    #[test]
    fn tiers_order_richest_to_leanest() {
        assert!(RenderTier::Ultra < RenderTier::High);
        assert!(RenderTier::High < RenderTier::Web);
    }
}
