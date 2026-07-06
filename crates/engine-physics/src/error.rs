//! Physics layer errors: collision, query, and controller operations.
//!
//! All errors are typed and fail-loud — no silent fallbacks, no leaked internals.

use thiserror::Error;

/// Physics operation errors.
#[derive(Debug, Error)]
pub enum PhysicsError {
  /// Invalid shape configuration (e.g., non-positive radius).
  #[error("Invalid shape: {0}")]
  InvalidShape(String),

  /// Ray/shape query failed (e.g., invalid direction).
  #[error("Invalid query: {0}")]
  InvalidQuery(String),

  /// Controller movement failed (e.g., invalid delta or floor state).
  #[error("Controller error: {0}")]
  ControllerError(String),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn error_display() {
    let err = PhysicsError::InvalidShape("radius must be positive".to_string());
    assert!(err.to_string().contains("Invalid shape"));
  }
}
