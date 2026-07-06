//! Typed, fail-loud errors for audio configuration.

use thiserror::Error;

/// An audio-layer error. Configuration is validated up front so a bad attenuation
/// curve or listener pose fails at construction rather than emitting NaNs into the
/// mix.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AudioError {
    /// A spatial/AoI parameter was out of range or non-finite.
    #[error("invalid audio configuration: {0}")]
    InvalidConfig(String),
}

impl AudioError {
    /// Build an [`AudioError::InvalidConfig`] from any message.
    #[must_use]
    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }
}
