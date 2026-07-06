//! Typed errors for audio operations.

use thiserror::Error;

/// Audio operation errors.
#[derive(Error, Debug, Clone)]
pub enum AudioError {
    /// Attempted to play a non-existent audio clip.
    #[error("audio clip not found: {0}")]
    ClipNotFound(String),

    /// Audio system not available (feature not enabled or platform limitation).
    #[error("audio system not available")]
    NotAvailable,

    /// Invalid audio source configuration.
    #[error("invalid audio source configuration: {0}")]
    InvalidConfig(String),

    /// Audio playback error.
    #[error("audio playback error: {0}")]
    PlaybackError(String),
}
