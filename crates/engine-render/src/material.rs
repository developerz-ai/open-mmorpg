//! glTF **metallic-roughness → PBR material** data mapping.
//!
//! Materials are [glTF 2.0 metallic-roughness PBR] loaded as *data*, never
//! hand-authored shaders per asset. This module is the pure, headless half of that
//! contract: parse the glTF factors ([`GltfMetallicRoughness`]) into a validated,
//! GPU-free [`PbrMaterial`] — fail loud on a non-finite factor, clamp the rest into
//! their legal ranges. A CI agent validates AI-generated material data exactly as
//! the rendered client does.
//!
//! Under the `render` feature, [`PbrMaterial::to_standard_material`] maps that data
//! onto Bevy's [`StandardMaterial`](bevy_pbr::StandardMaterial). The conversion is a
//! plain field copy — it constructs no GPU resource; the render app uploads it
//! later — so it is one material set feeding every [tier](crate::RenderTier), with
//! no per-asset shader fork.
//!
//! [glTF 2.0 metallic-roughness PBR]: <https://www.khronos.org/gltf/>

use bevy_reflect::{std_traits::ReflectDefault, Reflect};

use crate::error::RenderError;

/// How a surface's alpha is composited — the three glTF core alpha modes. Mask
/// carries its own cutoff so the invalid "cutoff without a mask" state is
/// unrepresentable.
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub enum MaterialAlphaMode {
    /// Fully opaque; alpha is ignored.
    #[default]
    Opaque,
    /// Alpha-tested: fragments with `alpha < cutoff` are discarded.
    Mask(f32),
    /// Alpha-blended over the background.
    Blend,
}

#[cfg(feature = "render")]
impl MaterialAlphaMode {
    /// Map to Bevy's [`AlphaMode`](bevy_material::AlphaMode). Direct — the three
    /// glTF core modes are a subset of Bevy's.
    #[must_use]
    pub fn to_bevy(self) -> bevy_material::AlphaMode {
        match self {
            MaterialAlphaMode::Opaque => bevy_material::AlphaMode::Opaque,
            MaterialAlphaMode::Mask(cutoff) => bevy_material::AlphaMode::Mask(cutoff),
            MaterialAlphaMode::Blend => bevy_material::AlphaMode::Blend,
        }
    }
}

/// The raw glTF `KHR_materials_pbrMetallicRoughness` factors (plus the two common
/// extensions we honor), as parsed from an asset. [`Default`] is the glTF spec
/// default material: opaque white, fully metallic and rough.
///
/// This is the *input* to mapping; [`PbrMaterial::from_gltf`] validates and clamps
/// it. Textures are resolved by the asset pipeline, not here — this is the factor
/// data that has no GPU dependency.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub struct GltfMetallicRoughness {
    /// Linear-space base color RGBA (`baseColorFactor`).
    pub base_color_factor: [f32; 4],
    /// Metalness (`metallicFactor`), in `[0, 1]`.
    pub metallic_factor: f32,
    /// Perceptual roughness (`roughnessFactor`), in `[0, 1]`.
    pub roughness_factor: f32,
    /// Linear-space emissive RGB (`emissiveFactor`), before strength.
    pub emissive_factor: [f32; 3],
    /// Emissive multiplier (`KHR_materials_emissive_strength`); `1.0` = no boost.
    pub emissive_strength: f32,
    /// Alpha compositing mode + cutoff (`alphaMode` / `alphaCutoff`).
    pub alpha_mode: MaterialAlphaMode,
    /// Render both faces (`doubleSided`) — disables back-face culling.
    pub double_sided: bool,
    /// Skip lighting (`KHR_materials_unlit`) — emit base color directly.
    pub unlit: bool,
}

impl Default for GltfMetallicRoughness {
    fn default() -> Self {
        // The glTF 2.0 spec defaults: opaque white, fully metallic + rough.
        Self {
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            emissive_factor: [0.0, 0.0, 0.0],
            emissive_strength: 1.0,
            alpha_mode: MaterialAlphaMode::Opaque,
            double_sided: false,
            unlit: false,
        }
    }
}

/// Validated, GPU-free PBR material data — the mapped result of
/// [`GltfMetallicRoughness`]. Every factor is finite and in range; the render layer
/// turns it into a [`StandardMaterial`](bevy_pbr::StandardMaterial) verbatim.
///
/// [`Default`] mirrors Bevy's [`StandardMaterial`](bevy_pbr::StandardMaterial)
/// default: opaque white, non-metallic, mid-roughness.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub struct PbrMaterial {
    /// Linear-space base color RGBA, each channel in `[0, 1]`.
    pub base_color: [f32; 4],
    /// Linear-space emissive RGB (`emissive_factor * emissive_strength`), each
    /// channel `>= 0` (may exceed `1.0` for HDR emission).
    pub emissive: [f32; 3],
    /// Metalness in `[0, 1]`.
    pub metallic: f32,
    /// Perceptual roughness in `[0, 1]`.
    pub perceptual_roughness: f32,
    /// Alpha compositing mode (Mask cutoff clamped to `[0, 1]`).
    pub alpha_mode: MaterialAlphaMode,
    /// Render both faces (no back-face culling).
    pub double_sided: bool,
    /// Skip lighting; emit base color directly.
    pub unlit: bool,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            emissive: [0.0, 0.0, 0.0],
            metallic: 0.0,
            perceptual_roughness: 0.5,
            alpha_mode: MaterialAlphaMode::Opaque,
            double_sided: false,
            unlit: false,
        }
    }
}

impl PbrMaterial {
    /// Validate and map glTF metallic-roughness factors into PBR material data.
    ///
    /// Non-finite factors are **rejected** (a broken asset must be fixed, not
    /// silently rendered as garbage); in-range violations are **clamped** to the
    /// legal range, matching how conformant glTF importers treat authored data.
    /// Emissive is `emissive_factor * emissive_strength`, floored at `0` but left
    /// unbounded above for HDR emission.
    ///
    /// # Errors
    /// [`RenderError::InvalidMaterialParameter`] naming the first non-finite factor.
    pub fn from_gltf(src: &GltfMetallicRoughness) -> Result<Self, RenderError> {
        let base_color = finite_array(src.base_color_factor, "base_color_factor")?.map(clamp01);
        let metallic = clamp01(finite(src.metallic_factor, "metallic_factor")?);
        let perceptual_roughness = clamp01(finite(src.roughness_factor, "roughness_factor")?);

        let strength = finite(src.emissive_strength, "emissive_strength")?;
        let emissive = finite_array(src.emissive_factor, "emissive_factor")?
            .map(|channel| (channel * strength).max(0.0));

        let alpha_mode = match src.alpha_mode {
            MaterialAlphaMode::Mask(cutoff) => {
                MaterialAlphaMode::Mask(clamp01(finite(cutoff, "alpha_cutoff")?))
            }
            direct => direct,
        };

        Ok(Self {
            base_color,
            emissive,
            metallic,
            perceptual_roughness,
            alpha_mode,
            double_sided: src.double_sided,
            unlit: src.unlit,
        })
    }
}

#[cfg(feature = "render")]
impl PbrMaterial {
    /// Build a Bevy [`StandardMaterial`](bevy_pbr::StandardMaterial) from this
    /// validated data. A pure field copy — no GPU resource is created here; the
    /// render app uploads the material later. glTF factors are linear, so the base
    /// color is set in linear space.
    #[must_use]
    pub fn to_standard_material(&self) -> bevy_pbr::StandardMaterial {
        use bevy_color::{Color, LinearRgba};
        use bevy_render::render_resource::Face;

        let [r, g, b, a] = self.base_color;
        let [er, eg, eb] = self.emissive;
        bevy_pbr::StandardMaterial {
            base_color: Color::linear_rgba(r, g, b, a),
            emissive: LinearRgba::rgb(er, eg, eb),
            metallic: self.metallic,
            perceptual_roughness: self.perceptual_roughness,
            alpha_mode: self.alpha_mode.to_bevy(),
            double_sided: self.double_sided,
            // glTF `doubleSided` = render both faces = cull neither.
            cull_mode: if self.double_sided {
                None
            } else {
                Some(Face::Back)
            },
            unlit: self.unlit,
            ..Default::default()
        }
    }
}

/// Reject a non-finite factor loud, naming the offending parameter.
fn finite(value: f32, parameter: &str) -> Result<f32, RenderError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(RenderError::InvalidMaterialParameter {
            parameter: parameter.to_owned(),
            detail: format!("expected a finite number, got {value}"),
        })
    }
}

/// Reject if any component is non-finite, naming the offending index.
fn finite_array<const N: usize>(
    values: [f32; N],
    parameter: &str,
) -> Result<[f32; N], RenderError> {
    for (index, &value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(RenderError::InvalidMaterialParameter {
                parameter: format!("{parameter}[{index}]"),
                detail: format!("expected a finite number, got {value}"),
            });
        }
    }
    Ok(values)
}

/// Clamp to the `[0, 1]` range every normalized PBR factor lives in.
fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_gltf_maps_to_metallic_rough_white() {
        // glTF spec default: opaque white, fully metallic + rough.
        let mat = PbrMaterial::from_gltf(&GltfMetallicRoughness::default())
            .expect("the spec-default material must map cleanly");
        assert_eq!(mat.base_color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(mat.metallic, 1.0);
        assert_eq!(mat.perceptual_roughness, 1.0);
        assert_eq!(mat.emissive, [0.0, 0.0, 0.0]);
        assert_eq!(mat.alpha_mode, MaterialAlphaMode::Opaque);
        assert!(!mat.double_sided);
        assert!(!mat.unlit);
    }

    #[test]
    fn factors_pass_through_when_in_range() {
        let src = GltfMetallicRoughness {
            base_color_factor: [0.2, 0.4, 0.6, 0.8],
            metallic_factor: 0.25,
            roughness_factor: 0.75,
            alpha_mode: MaterialAlphaMode::Blend,
            double_sided: true,
            unlit: true,
            ..Default::default()
        };
        let mat = PbrMaterial::from_gltf(&src).expect("in-range factors map cleanly");
        assert_eq!(mat.base_color, [0.2, 0.4, 0.6, 0.8]);
        assert_eq!(mat.metallic, 0.25);
        assert_eq!(mat.perceptual_roughness, 0.75);
        assert_eq!(mat.alpha_mode, MaterialAlphaMode::Blend);
        assert!(mat.double_sided);
        assert!(mat.unlit);
    }

    #[test]
    fn out_of_range_factors_are_clamped() {
        let src = GltfMetallicRoughness {
            base_color_factor: [-0.5, 2.0, 0.5, 1.5],
            metallic_factor: 3.0,
            roughness_factor: -1.0,
            alpha_mode: MaterialAlphaMode::Mask(9.0),
            ..Default::default()
        };
        let mat = PbrMaterial::from_gltf(&src).expect("out-of-range clamps, does not fail");
        assert_eq!(mat.base_color, [0.0, 1.0, 0.5, 1.0]);
        assert_eq!(mat.metallic, 1.0);
        assert_eq!(mat.perceptual_roughness, 0.0);
        assert_eq!(mat.alpha_mode, MaterialAlphaMode::Mask(1.0));
    }

    #[test]
    fn emissive_scales_by_strength_and_stays_hdr() {
        let src = GltfMetallicRoughness {
            emissive_factor: [0.5, 0.25, 0.0],
            emissive_strength: 4.0,
            ..Default::default()
        };
        let mat = PbrMaterial::from_gltf(&src).expect("valid emissive maps cleanly");
        // Scaled past 1.0 and left unbounded above for HDR emission.
        assert_eq!(mat.emissive, [2.0, 1.0, 0.0]);
    }

    #[test]
    fn negative_emissive_is_floored_at_zero() {
        let src = GltfMetallicRoughness {
            emissive_factor: [1.0, 1.0, 1.0],
            emissive_strength: -2.0,
            ..Default::default()
        };
        let mat = PbrMaterial::from_gltf(&src).expect("negative strength clamps, does not fail");
        assert_eq!(mat.emissive, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn non_finite_factor_is_rejected_loudly() {
        let src = GltfMetallicRoughness {
            metallic_factor: f32::NAN,
            ..Default::default()
        };
        let err = PbrMaterial::from_gltf(&src).expect_err("NaN factor must fail loud");
        assert_eq!(
            err,
            RenderError::InvalidMaterialParameter {
                parameter: "metallic_factor".to_owned(),
                detail: "expected a finite number, got NaN".to_owned(),
            }
        );
    }

    #[test]
    fn non_finite_array_component_names_its_index() {
        let src = GltfMetallicRoughness {
            base_color_factor: [1.0, f32::INFINITY, 1.0, 1.0],
            ..Default::default()
        };
        let err = PbrMaterial::from_gltf(&src).expect_err("infinite channel must fail loud");
        let RenderError::InvalidMaterialParameter { parameter, .. } = err else {
            panic!("expected InvalidMaterialParameter, got {err:?}");
        };
        assert_eq!(parameter, "base_color_factor[1]");
    }

    #[test]
    fn default_pbr_material_matches_standard_material_defaults() {
        // The pure default must mirror Bevy's StandardMaterial default so an
        // unset material renders identically headless and headful.
        let mat = PbrMaterial::default();
        assert_eq!(mat.base_color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(mat.metallic, 0.0);
        assert_eq!(mat.perceptual_roughness, 0.5);
        assert_eq!(mat.alpha_mode, MaterialAlphaMode::Opaque);
    }
}

#[cfg(all(test, feature = "render"))]
mod render_tests {
    use super::*;

    // These build a `StandardMaterial` struct (pure data — no GPU device, no
    // window) and assert the field mapping. Safe under `--all-features` in a
    // headless CI runner.

    #[test]
    fn maps_pbr_data_onto_standard_material_fields() {
        use bevy_color::{Color, LinearRgba};

        let mat = PbrMaterial {
            base_color: [0.1, 0.2, 0.3, 0.4],
            emissive: [1.5, 0.0, 0.0],
            metallic: 0.6,
            perceptual_roughness: 0.3,
            alpha_mode: MaterialAlphaMode::Mask(0.25),
            double_sided: false,
            unlit: true,
        };
        let std = mat.to_standard_material();
        assert_eq!(std.base_color, Color::linear_rgba(0.1, 0.2, 0.3, 0.4));
        assert_eq!(std.emissive, LinearRgba::rgb(1.5, 0.0, 0.0));
        assert_eq!(std.metallic, 0.6);
        assert_eq!(std.perceptual_roughness, 0.3);
        assert_eq!(std.alpha_mode, bevy_material::AlphaMode::Mask(0.25));
        assert!(std.unlit);
    }

    #[test]
    fn single_sided_culls_back_faces_double_sided_culls_none() {
        use bevy_render::render_resource::Face;

        let single = PbrMaterial {
            double_sided: false,
            ..Default::default()
        }
        .to_standard_material();
        assert_eq!(single.cull_mode, Some(Face::Back));
        assert!(!single.double_sided);

        let double = PbrMaterial {
            double_sided: true,
            ..Default::default()
        }
        .to_standard_material();
        assert_eq!(double.cull_mode, None);
        assert!(double.double_sided);
    }

    #[test]
    fn alpha_modes_map_one_to_one() {
        assert_eq!(
            MaterialAlphaMode::Opaque.to_bevy(),
            bevy_material::AlphaMode::Opaque
        );
        assert_eq!(
            MaterialAlphaMode::Blend.to_bevy(),
            bevy_material::AlphaMode::Blend
        );
        assert_eq!(
            MaterialAlphaMode::Mask(0.5).to_bevy(),
            bevy_material::AlphaMode::Mask(0.5)
        );
    }
}
