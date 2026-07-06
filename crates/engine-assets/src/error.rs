//! Typed, fail-loud errors for the asset pipeline — manifest slot validation,
//! LOD chain construction and camera-position streaming.
//!
//! Manifests are authored by hand, by an artist tool or by the AI content
//! firehose, so they *will* be wrong sometimes. The contract (see
//! `docs/specs/game-engine/assets/README.md`) is that a missing or invalid asset
//! is a **visible error, never a silent blank**. Each variant names one distinct,
//! actionable failure so a tool or agent can react precisely rather than parse a
//! string.

use thiserror::Error;

/// Everything the asset pipeline refuses. Grouped by stage: manifest parse/slot
/// validation, then LOD-chain construction, then streaming.
#[derive(Debug, Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum AssetError {
    // ---- manifest parse / schema ----
    /// The manifest bytes are not valid JSON, or do not match the schema.
    #[error("failed to parse asset manifest: {0}")]
    Parse(String),

    /// The manifest targets an asset-API version this engine build cannot read.
    #[error("asset manifest api_version {found} is unsupported (engine expects {expected})")]
    UnsupportedApiVersion {
        /// Version this engine build understands.
        expected: u32,
        /// Version the manifest declared.
        found: u32,
    },

    /// A required string field on a slot is empty (e.g. a blank model path). An
    /// empty path can never resolve to a file, so it is rejected at load.
    #[error("slot `{slot}` declares an empty {field}")]
    EmptyField {
        /// Slot id (or `<unnamed>` when the id itself is the empty field).
        slot: String,
        /// Which field was empty (`id`, `model`, `rig`, `texture role`, …).
        field: &'static str,
    },

    /// Two slots share an id. Slot ids are the lookup key, so they must be unique.
    #[error("duplicate slot id `{id}`")]
    DuplicateSlot {
        /// The repeated slot id.
        id: String,
    },

    /// An asset path uses an extension outside the open-format allow-list.
    /// "Open formats only" is a hard rule — a proprietary container never enters
    /// the chain.
    #[error("slot `{slot}`: {kind} `{path}` is not an open format (allowed: {allowed})")]
    UnsupportedFormat {
        /// Slot id the offending path belongs to.
        slot: String,
        /// The rejected asset path.
        path: String,
        /// Which asset it was (`model`, `texture`, `rig`).
        kind: &'static str,
        /// Comma-separated allowed extensions.
        allowed: &'static str,
    },

    /// A skinned/animated slot (or candidate) has no rig. A character with no rig
    /// cannot animate, so the slot spec is invalid.
    #[error("slot `{slot}` is animated and requires a rig, but none was declared")]
    RigRequired {
        /// Slot id.
        slot: String,
    },

    /// A static slot (or candidate) declares a rig it can never use.
    #[error("slot `{slot}` is static and takes no rig, but one was declared")]
    UnexpectedRig {
        /// Slot id.
        slot: String,
    },

    /// A required texture role is absent — from a slot spec, or from a candidate
    /// bundle being fitted to a slot.
    #[error("slot `{slot}` is missing required texture role `{role}`")]
    MissingTexture {
        /// Slot id.
        slot: String,
        /// The missing PBR texture role (e.g. `albedo`).
        role: String,
    },

    /// A candidate offered to fill a slot has no model mesh.
    #[error("candidate for slot `{slot}` supplies no model")]
    MissingModel {
        /// Slot id the candidate was tested against.
        slot: String,
    },

    // ---- level of detail ----
    /// A LOD chain was built with no mesh tiers — there is nothing to draw.
    #[error("LOD chain has no mesh tiers")]
    EmptyLodChain,

    /// LOD screen-size thresholds are not strictly descending (or contain a
    /// non-finite value). Tier 0 is the finest (largest threshold); each coarser
    /// tier must trigger at a strictly smaller screen size.
    #[error("LOD thresholds must be finite and strictly descending; violated at index {index}")]
    LodNotDescending {
        /// Index `i` where `thresholds[i]` is not strictly greater than
        /// `thresholds[i + 1]`, or where a value is non-finite.
        index: usize,
    },

    /// The cull screen-size is not in `[0, coarsest_threshold)`; below the
    /// coarsest tier there would be no valid range for imposter/cull.
    #[error("LOD cull {cull} must be finite and in [0, {min_threshold})")]
    LodCull {
        /// The rejected cull size.
        cull: f32,
        /// The coarsest (smallest) mesh-tier threshold.
        min_threshold: f32,
    },

    // ---- streaming ----
    /// A streaming-grid parameter is non-positive or non-finite.
    #[error("invalid streaming config: {field} must be finite and positive")]
    InvalidStreamingConfig {
        /// The offending field (`tile_size`, `view_radius`, `budget`).
        field: &'static str,
    },

    /// A single tile's memory cost exceeds the entire streaming budget, so it can
    /// never be resident regardless of eviction — a content bug, not back-pressure.
    #[error("tile ({x}, {y}) costs {cost} bytes, over the whole {budget}-byte budget")]
    TileExceedsBudget {
        /// Tile x coordinate.
        x: i32,
        /// Tile y coordinate.
        y: i32,
        /// The tile's memory cost in bytes.
        cost: u64,
        /// The total streaming budget in bytes.
        budget: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_reads_cleanly() {
        let err = AssetError::Parse("unexpected `}`".to_owned());
        assert_eq!(
            err.to_string(),
            "failed to parse asset manifest: unexpected `}`"
        );
    }

    #[test]
    fn api_version_names_both_numbers() {
        let msg = AssetError::UnsupportedApiVersion {
            expected: 1,
            found: 7,
        }
        .to_string();
        assert!(msg.contains('1') && msg.contains('7'), "got: {msg}");
    }

    #[test]
    fn unsupported_format_lists_allowed() {
        let msg = AssetError::UnsupportedFormat {
            slot: "aurin-body".to_owned(),
            path: "body.fbx".to_owned(),
            kind: "model",
            allowed: "gltf, glb",
        }
        .to_string();
        assert!(msg.contains("body.fbx"), "got: {msg}");
        assert!(msg.contains("gltf, glb"), "got: {msg}");
    }

    #[test]
    fn tile_exceeds_budget_reports_coords_and_sizes() {
        let msg = AssetError::TileExceedsBudget {
            x: -3,
            y: 4,
            cost: 2048,
            budget: 1024,
        }
        .to_string();
        assert!(msg.contains("(-3, 4)"), "got: {msg}");
        assert!(msg.contains("2048") && msg.contains("1024"), "got: {msg}");
    }

    #[test]
    fn errors_are_comparable_for_precise_test_assertions() {
        assert_eq!(AssetError::EmptyLodChain, AssetError::EmptyLodChain);
        assert_ne!(
            AssetError::LodNotDescending { index: 0 },
            AssetError::LodNotDescending { index: 1 }
        );
    }
}
