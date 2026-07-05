//! Typed errors for the script sandbox.
//!
//! Every failure a script can produce maps to a stable [`ClientCode`] so the
//! host can react uniformly. Content faults (bad module, missing/mistyped
//! export) surface as [`ClientCode::BadRequest`]; a script that is sandbox-killed
//! (fuel exhausted, trap) is contained and reported as [`ClientCode::Internal`];
//! a capability the host refuses is [`ClientCode::Forbidden`]. Detail strings are
//! for server logs only — they never carry secrets.

use omm_errors::ClientCode;

/// A failure raised while compiling or running an untrusted content script.
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    /// The module bytes were not valid WebAssembly, or used a disabled feature.
    #[error("script failed to compile: {0}")]
    CompileFailed(String),
    /// The module could not be linked/instantiated — typically because it
    /// imports a host function outside the capability API.
    #[error("script could not be instantiated: {0}")]
    Instantiation(String),
    /// The requested entry function is not exported by the module.
    #[error("script does not export '{0}'")]
    MissingExport(String),
    /// The entry export exists but has the wrong signature for the ABI.
    #[error("script export '{0}' has an unexpected signature")]
    BadSignature(String),
    /// The script exhausted its fuel budget — starved, not allowed to hang.
    #[error("script exhausted its fuel budget")]
    OutOfFuel,
    /// The script trapped at runtime (unreachable, div-by-zero, bad memory …).
    #[error("script trapped: {0}")]
    Trap(String),
    /// A capability call was refused by the host (e.g. an invalid grant).
    #[error("host denied a capability call: {0}")]
    CapabilityDenied(String),
    /// The host could not configure the sandbox itself (setup fault, not the
    /// script's fault).
    #[error("sandbox host error: {0}")]
    Host(String),
}

impl ScriptError {
    /// The stable, wire-safe code for this error.
    #[must_use]
    pub const fn code(&self) -> ClientCode {
        match self {
            Self::CompileFailed(_) | Self::MissingExport(_) | Self::BadSignature(_) => {
                ClientCode::BadRequest
            }
            Self::CapabilityDenied(_) => ClientCode::Forbidden,
            Self::OutOfFuel | Self::Trap(_) | Self::Instantiation(_) | Self::Host(_) => {
                ClientCode::Internal
            }
        }
    }
}

/// Convenience alias for fallible sandbox operations.
pub type ScriptResult<T> = Result<T, ScriptError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_faults_map_to_bad_request() {
        assert_eq!(
            ScriptError::CompileFailed("x".into()).code(),
            ClientCode::BadRequest
        );
        assert_eq!(
            ScriptError::MissingExport("run".into()).code(),
            ClientCode::BadRequest
        );
        assert_eq!(
            ScriptError::BadSignature("run".into()).code(),
            ClientCode::BadRequest
        );
    }

    #[test]
    fn contained_faults_map_to_internal() {
        assert_eq!(ScriptError::OutOfFuel.code(), ClientCode::Internal);
        assert_eq!(
            ScriptError::Trap("boom".into()).code(),
            ClientCode::Internal
        );
    }

    #[test]
    fn denied_capability_is_forbidden() {
        assert_eq!(
            ScriptError::CapabilityDenied("no".into()).code(),
            ClientCode::Forbidden
        );
    }
}
