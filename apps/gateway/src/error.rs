//! Typed gateway errors.
//!
//! Every error maps to a stable [`ClientCode`] and an HTTP status; the JSON body
//! carries only the code and a fixed, safe message. Internal detail and
//! credentials never cross the boundary
//! (`docs/specs/game-server/security/README.md`).

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use omm_errors::ClientCode;
use serde::Serialize;

/// Why a token or credential check failed. Shared by [`crate::session`]
/// (token verification) and credential verifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AuthError {
    /// The token was structurally invalid (bad shape, base64, or length).
    #[error("malformed token")]
    Malformed,
    /// The MAC did not match — the token was forged or tampered with.
    #[error("bad signature")]
    BadSignature,
    /// The token is past its expiry.
    #[error("expired token")]
    Expired,
    /// The presented credentials did not authenticate.
    #[error("invalid credentials")]
    InvalidCredentials,
    /// An internal fault (e.g. signer misconfiguration). No detail exposed.
    #[error("internal auth error")]
    Internal,
}

/// The top-level error returned by gateway HTTP handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GatewayError {
    /// An authentication/authorization failure.
    #[error(transparent)]
    Auth(#[from] AuthError),
    /// The caller was rate-limited at the edge.
    #[error("rate limited")]
    RateLimited,
}

impl GatewayError {
    /// The stable, wire-safe client code for this error.
    #[must_use]
    pub const fn code(&self) -> ClientCode {
        match self {
            Self::Auth(AuthError::Malformed) => ClientCode::BadRequest,
            Self::Auth(
                AuthError::BadSignature | AuthError::Expired | AuthError::InvalidCredentials,
            ) => ClientCode::Unauthenticated,
            Self::Auth(AuthError::Internal) => ClientCode::Internal,
            Self::RateLimited => ClientCode::TooManyRequests,
        }
    }

    /// The HTTP status this error maps to.
    #[must_use]
    pub const fn status(&self) -> StatusCode {
        match self.code() {
            ClientCode::BadRequest => StatusCode::BAD_REQUEST,
            ClientCode::Unauthenticated => StatusCode::UNAUTHORIZED,
            ClientCode::Forbidden => StatusCode::FORBIDDEN,
            ClientCode::NotFound => StatusCode::NOT_FOUND,
            ClientCode::Conflict => StatusCode::CONFLICT,
            ClientCode::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ClientCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// The JSON error envelope sent to clients: a stable code and a safe message.
#[derive(Debug, Serialize)]
struct ErrorBody {
    code: u16,
    message: &'static str,
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        // A fixed, non-leaking message per code — never echoes the request or a
        // cause chain.
        let message = match self.code() {
            ClientCode::BadRequest => "bad request",
            ClientCode::Unauthenticated => "unauthenticated",
            ClientCode::Forbidden => "forbidden",
            ClientCode::NotFound => "not found",
            ClientCode::Conflict => "conflict",
            ClientCode::TooManyRequests => "too many requests",
            ClientCode::Internal => "internal error",
        };
        let body = ErrorBody {
            code: self.code().as_u16(),
            message,
        };
        (self.status(), Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_errors_map_to_stable_codes() {
        assert_eq!(
            GatewayError::Auth(AuthError::Malformed).code(),
            ClientCode::BadRequest
        );
        assert_eq!(
            GatewayError::Auth(AuthError::BadSignature).code(),
            ClientCode::Unauthenticated
        );
        assert_eq!(
            GatewayError::Auth(AuthError::Expired).code(),
            ClientCode::Unauthenticated
        );
        assert_eq!(
            GatewayError::Auth(AuthError::InvalidCredentials).code(),
            ClientCode::Unauthenticated
        );
        assert_eq!(
            GatewayError::RateLimited.code(),
            ClientCode::TooManyRequests
        );
    }

    #[test]
    fn statuses_match_codes() {
        assert_eq!(
            GatewayError::RateLimited.status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            GatewayError::Auth(AuthError::Expired).status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            GatewayError::Auth(AuthError::Malformed).status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn from_auth_error_wraps() {
        let e: GatewayError = AuthError::Expired.into();
        assert_eq!(e, GatewayError::Auth(AuthError::Expired));
    }
}
