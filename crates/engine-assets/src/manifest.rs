//! Manifest-driven asset slots — parse `manifest.json` and validate each slot
//! spec (model + texture set + rig) fail-loud.
//!
//! Each race/class/item declares a **slot spec**: the model mesh, the PBR texture
//! roles and (for animated things) the rig it needs. The AI content firehose then
//! only has to *fit the slot* — a generated bundle is accepted iff it supplies
//! every piece the slot declares ([`AssetSlot::accepts`]). No bespoke importer per
//! asset; the manifest is the whole contract. → `docs/specs/game-engine/assets/README.md`.
//!
//! Everything here is pure and headless: bytes in, typed schema or [`AssetError`]
//! out. No GPU, no window, no file IO — the render head and a CI agent validate a
//! manifest identically.
//!
//! ```json
//! {
//!   "id": "open-mmorpg.characters",
//!   "version": "0.1.0",
//!   "api_version": 1,
//!   "slots": [
//!     {
//!       "id": "aurin-body",
//!       "kind": "character",
//!       "model": "aurin_body.gltf",
//!       "textures": { "albedo": "aurin_albedo.ktx2", "normal": "aurin_normal.ktx2" },
//!       "rig": "aurin_rig.gltf"
//!     }
//!   ]
//! }
//! ```

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::error::AssetError;

/// Asset-API version this engine build reads. Bumped when the manifest schema
/// changes incompatibly; a mismatch is a fail-loud [`AssetError::UnsupportedApiVersion`].
pub const ENGINE_API_VERSION: u32 = 1;

/// The one texture role every renderable slot must declare — the PBR base colour
/// (glTF `baseColorTexture`). A material with no albedo is a bug, not a style.
pub const BASE_COLOR_ROLE: &str = "albedo";

/// Open model/rig container formats (glTF is the mesh/material/anim source of truth).
const MODEL_FORMATS: &[&str] = &["gltf", "glb"];
/// Human-readable form of [`MODEL_FORMATS`] for error messages — keep in sync.
const MODEL_FORMATS_STR: &str = "gltf, glb";
/// Open, GPU-friendly texture containers (KTX2/Basis shipping, PNG/EXR authoring).
const TEXTURE_FORMATS: &[&str] = &["ktx2", "basis", "dds", "png", "exr"];
/// Human-readable form of [`TEXTURE_FORMATS`] for error messages — keep in sync.
const TEXTURE_FORMATS_STR: &str = "ktx2, basis, dds, png, exr";

/// What a slot represents. The kind fixes whether a rig is required: only skinned
/// things animate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotKind {
    /// Skinned, animated content — races, classes, creatures. **Requires a rig.**
    Character,
    /// Static content — props, items, scenery. **Takes no rig.**
    Prop,
}

impl SlotKind {
    /// Whether a slot of this kind must declare an animation rig.
    #[must_use]
    pub fn requires_rig(self) -> bool {
        matches!(self, SlotKind::Character)
    }
}

/// One asset slot spec: the model, PBR texture roles and optional rig a piece of
/// content must supply to fill this slot.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AssetSlot {
    /// Unique slot id (the lookup key), e.g. `aurin-body`.
    pub id: String,
    /// Whether this slot is animated (needs a rig) or static.
    pub kind: SlotKind,
    /// Model mesh path — glTF/glB, relative to the bundle root.
    pub model: String,
    /// PBR texture roles → file paths. Must include [`BASE_COLOR_ROLE`].
    #[serde(default)]
    pub textures: BTreeMap<String, String>,
    /// Animation rig path — required iff [`SlotKind::requires_rig`], else omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rig: Option<String>,
}

impl AssetSlot {
    /// Validate this slot spec is internally well-formed and open-format only.
    ///
    /// # Errors
    /// The first violation found: empty id/model/texture entry, a non-open format,
    /// a missing [`BASE_COLOR_ROLE`], or a rig that is required-but-absent /
    /// present-but-forbidden.
    pub fn validate(&self) -> Result<(), AssetError> {
        if self.id.is_empty() {
            return Err(AssetError::EmptyField {
                slot: "<unnamed>".to_owned(),
                field: "id",
            });
        }
        if self.model.is_empty() {
            return Err(AssetError::EmptyField {
                slot: self.id.clone(),
                field: "model",
            });
        }
        check_format(
            &self.id,
            &self.model,
            "model",
            MODEL_FORMATS,
            MODEL_FORMATS_STR,
        )?;

        for (role, path) in &self.textures {
            if role.is_empty() {
                return Err(AssetError::EmptyField {
                    slot: self.id.clone(),
                    field: "texture role",
                });
            }
            if path.is_empty() {
                return Err(AssetError::EmptyField {
                    slot: self.id.clone(),
                    field: "texture path",
                });
            }
            check_format(
                &self.id,
                path,
                "texture",
                TEXTURE_FORMATS,
                TEXTURE_FORMATS_STR,
            )?;
        }
        if !self.textures.contains_key(BASE_COLOR_ROLE) {
            return Err(AssetError::MissingTexture {
                slot: self.id.clone(),
                role: BASE_COLOR_ROLE.to_owned(),
            });
        }

        self.validate_rig()
    }

    /// Enforce the rig ⇔ kind invariant: characters need a rig, props forbid one.
    fn validate_rig(&self) -> Result<(), AssetError> {
        match (&self.rig, self.kind.requires_rig()) {
            (Some(rig), true) => {
                if rig.is_empty() {
                    return Err(AssetError::EmptyField {
                        slot: self.id.clone(),
                        field: "rig",
                    });
                }
                check_format(&self.id, rig, "rig", MODEL_FORMATS, MODEL_FORMATS_STR)
            }
            (None, true) => Err(AssetError::RigRequired {
                slot: self.id.clone(),
            }),
            (Some(_), false) => Err(AssetError::UnexpectedRig {
                slot: self.id.clone(),
            }),
            (None, false) => Ok(()),
        }
    }

    /// Whether `candidate` (e.g. an AI-generated bundle) *fits this slot*: it
    /// supplies a model, every texture role the slot declares, and a rig exactly
    /// when the slot needs one. This is the "content only has to fit the slot" gate.
    ///
    /// # Errors
    /// The first missing/extra piece: no model, an absent texture role, or a rig
    /// that is required-but-absent / present-but-forbidden.
    pub fn accepts(&self, candidate: &SlotCandidate) -> Result<(), AssetError> {
        if candidate.model.is_none() {
            return Err(AssetError::MissingModel {
                slot: self.id.clone(),
            });
        }
        for role in self.textures.keys() {
            if !candidate.textures.contains(role) {
                return Err(AssetError::MissingTexture {
                    slot: self.id.clone(),
                    role: role.clone(),
                });
            }
        }
        match (candidate.rig.is_some(), self.kind.requires_rig()) {
            (false, true) => Err(AssetError::RigRequired {
                slot: self.id.clone(),
            }),
            (true, false) => Err(AssetError::UnexpectedRig {
                slot: self.id.clone(),
            }),
            _ => Ok(()),
        }
    }
}

/// A candidate asset bundle offered to fill a slot — the shape the AI/artist
/// pipeline produces, checked against a slot spec by [`AssetSlot::accepts`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SlotCandidate {
    /// The model mesh path the candidate supplies, if any.
    pub model: Option<String>,
    /// The texture roles the candidate supplies (role names, not paths).
    pub textures: BTreeSet<String>,
    /// The rig path the candidate supplies, if any.
    pub rig: Option<String>,
}

/// The root of an asset bundle: metadata plus the slot specs it defines.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AssetManifest {
    /// Reverse-DNS bundle id, e.g. `open-mmorpg.characters`.
    pub id: String,
    /// Bundle content version (semver string; opaque to the engine).
    pub version: String,
    /// Asset-API schema version — must equal [`ENGINE_API_VERSION`].
    pub api_version: u32,
    /// The slot specs this bundle defines.
    #[serde(default)]
    pub slots: Vec<AssetSlot>,
}

impl AssetManifest {
    /// Parse and validate a manifest from raw JSON bytes.
    ///
    /// # Errors
    /// [`AssetError::Parse`] if the bytes are not valid JSON for the schema, then
    /// whatever [`AssetManifest::validate`] rejects.
    pub fn from_json(bytes: &[u8]) -> Result<Self, AssetError> {
        let manifest: AssetManifest =
            serde_json::from_slice(bytes).map_err(|e| AssetError::Parse(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate api-version compatibility, every slot spec, and slot-id uniqueness.
    ///
    /// # Errors
    /// [`AssetError::UnsupportedApiVersion`], any per-slot error from
    /// [`AssetSlot::validate`], or [`AssetError::DuplicateSlot`].
    pub fn validate(&self) -> Result<(), AssetError> {
        if self.api_version != ENGINE_API_VERSION {
            return Err(AssetError::UnsupportedApiVersion {
                expected: ENGINE_API_VERSION,
                found: self.api_version,
            });
        }
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for slot in &self.slots {
            slot.validate()?;
            if !seen.insert(slot.id.as_str()) {
                return Err(AssetError::DuplicateSlot {
                    id: slot.id.clone(),
                });
            }
        }
        Ok(())
    }

    /// Look up a slot spec by id.
    #[must_use]
    pub fn slot(&self, id: &str) -> Option<&AssetSlot> {
        self.slots.iter().find(|slot| slot.id == id)
    }
}

/// Reject a path whose extension is not in `allowed`.
fn check_format(
    slot: &str,
    path: &str,
    kind: &'static str,
    allowed: &[&str],
    allowed_str: &'static str,
) -> Result<(), AssetError> {
    let ok = extension(path).is_some_and(|ext| allowed.contains(&ext.as_str()));
    if ok {
        Ok(())
    } else {
        Err(AssetError::UnsupportedFormat {
            slot: slot.to_owned(),
            path: path.to_owned(),
            kind,
            allowed: allowed_str,
        })
    }
}

/// Lower-cased file extension after the final `.`, or `None` when there is no
/// (non-empty) extension. `"a.b.KTX2"` → `Some("ktx2")`, `"body"`/`"body."` → `None`.
fn extension(path: &str) -> Option<String> {
    path.rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
}

#[cfg(test)]
mod tests;
