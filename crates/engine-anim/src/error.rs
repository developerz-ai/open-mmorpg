//! Animation-layer errors: invalid clip ID, unsupported topology, blend evaluation failure.

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("animation error: {0}")]
pub struct AnimError(String);

impl AnimError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}
