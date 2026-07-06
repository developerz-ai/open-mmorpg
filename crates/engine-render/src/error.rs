//! Typed, fail-loud errors for the render pipeline — tier selection, material
//! mapping, CSM cascade computation, and capability detection.
//!
//! Render errors are actionable: a missing capability, an unsupported material
//! format, or a malformed CSM configuration is reported explicitly so tools and
//! agents can respond precisely rather than parse a string.

use thiserror::Error;

/// Everything the render pipeline refuses. Grouped by stage: capability
/// detection, then material mapping, then CSM setup.
#[derive(Debug, Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum RenderError {
    /// A required GPU capability is unavailable. Tier selection detected a gap.
    #[error("missing GPU capability: {capability}")]
    MissingCapability {
        /// The unsupported feature (e.g. `compute_shader`, `multi_sampled_rendering`).
        capability: String,
    },

    /// A glTF material format is not recognized or cannot be mapped to
    /// `StandardMaterial`.
    #[error("unsupported material format: {format}")]
    UnsupportedMaterialFormat {
        /// The glTF material extension or structure that failed (e.g. `KHR_transmission`).
        format: String,
    },

    /// CSM cascade parameters are invalid (e.g. non-positive split distance,
    /// cascade count out of bounds).
    #[error("invalid CSM configuration: {detail}")]
    InvalidCsmConfig {
        /// Description of the invalid parameter.
        detail: String,
    },

    /// Frame instrumentation (frame time, draw count) is out of expected bounds,
    /// signaling a performance anomaly or a misconfigured budget.
    #[error("frame instrumentation anomaly: {detail}")]
    InstrumentationAnomaly {
        /// Description of the out-of-bounds reading.
        detail: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_capability_error_names_feature() {
        let err = RenderError::MissingCapability {
            capability: "compute_shader".to_owned(),
        };
        assert!(err.to_string().contains("compute_shader"), "got: {}", err);
    }

    #[test]
    fn unsupported_material_format_reports_extension() {
        let err = RenderError::UnsupportedMaterialFormat {
            format: "KHR_transmission".to_owned(),
        };
        assert!(err.to_string().contains("KHR_transmission"));
    }

    #[test]
    fn invalid_csm_config_describes_parameter() {
        let err = RenderError::InvalidCsmConfig {
            detail: "cascade_count out of bounds (0..=4)".to_owned(),
        };
        assert!(err.to_string().contains("cascade_count"));
    }

    #[test]
    fn errors_are_comparable() {
        assert_eq!(
            RenderError::MissingCapability {
                capability: "feature".to_owned()
            },
            RenderError::MissingCapability {
                capability: "feature".to_owned()
            }
        );
    }
}
