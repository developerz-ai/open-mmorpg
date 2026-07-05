//! Shared, typed error enums for Open-MMORPG.
//!
//! Every error carries a **stable client code** (`ClientCode`) that is safe to
//! send to a client and safe to match on across versions. Human-readable detail
//! stays server-side — we never leak credentials or internal state to a client.

/// A stable, wire-safe error code. The numeric value is part of the client
/// contract: never renumber an existing variant, only append.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ClientCode {
    /// The request was malformed or failed validation.
    BadRequest = 1000,
    /// The caller is not authenticated.
    Unauthenticated = 1001,
    /// The caller is authenticated but not permitted.
    Forbidden = 1002,
    /// A referenced entity does not exist.
    NotFound = 1003,
    /// The action conflicts with current state (e.g. already claimed).
    Conflict = 1004,
    /// The server hit an internal fault. No detail is exposed.
    Internal = 2000,
}

impl ClientCode {
    /// The numeric code sent on the wire.
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self as u16
    }
}

/// The canonical error type crossing service boundaries.
///
/// `Internal` deliberately hides its cause from clients: the `String` is for
/// server logs only and must never contain secrets.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("forbidden")]
    Forbidden,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error")]
    Internal,
}

impl CoreError {
    /// The stable client code for this error — safe to send over the wire.
    #[must_use]
    pub const fn code(&self) -> ClientCode {
        match self {
            Self::BadRequest(_) => ClientCode::BadRequest,
            Self::Unauthenticated => ClientCode::Unauthenticated,
            Self::Forbidden => ClientCode::Forbidden,
            Self::NotFound(_) => ClientCode::NotFound,
            Self::Conflict(_) => ClientCode::Conflict,
            Self::Internal => ClientCode::Internal,
        }
    }
}

/// Convenience alias for fallible core operations.
pub type CoreResult<T> = Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable_and_distinct() {
        assert_eq!(ClientCode::BadRequest.as_u16(), 1000);
        assert_eq!(ClientCode::Internal.as_u16(), 2000);
        assert_ne!(ClientCode::NotFound, ClientCode::Conflict);
    }

    #[test]
    fn error_maps_to_expected_code() {
        assert_eq!(
            CoreError::Unauthenticated.code(),
            ClientCode::Unauthenticated
        );
        assert_eq!(
            CoreError::NotFound("char".into()).code(),
            ClientCode::NotFound
        );
    }

    #[test]
    fn internal_error_hides_detail() {
        // The public Display must not carry a cause string.
        assert_eq!(CoreError::Internal.to_string(), "internal error");
    }
}
