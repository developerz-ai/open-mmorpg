//! Credential verification.
//!
//! Login credentials are checked behind a trait so the real YugabyteDB-backed
//! verifier (via `crates/persistence`) can drop in later without touching the
//! HTTP layer. Server-authoritative: the client sends credentials, the server
//! decides the resulting [`AccountId`].

use omm_protocol::AccountId;
use serde::Deserialize;
use std::future::Future;

use crate::error::AuthError;
use crate::routing::fnv1a;

/// Client-supplied login credentials. Deserialized from the request body; never
/// logged.
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    /// The account username / login handle.
    pub username: String,
    /// The account password (or, later, an OAuth/session assertion).
    pub password: String,
}

/// Verifies credentials and resolves them to an authenticated account.
///
/// A trait (not a concrete DB call) so tests and the future persistence-backed
/// implementation share one seam. `async` via RPITIT — no `dyn`, so the
/// gateway state stays generic over the verifier.
pub trait CredentialVerifier: Send + Sync + 'static {
    /// Authenticate `creds`, returning the account on success.
    fn verify(
        &self,
        creds: &Credentials,
    ) -> impl Future<Output = Result<AccountId, AuthError>> + Send;
}

/// A development stand-in verifier.
///
/// Accepts any non-blank username/password pair and derives a stable
/// [`AccountId`] from the username hash. It performs **no** real authentication
/// and must be replaced by the persistence-backed verifier before production;
/// [`DevVerifier::new`] logs a warning to make that explicit.
#[derive(Debug, Clone, Copy, Default)]
pub struct DevVerifier;

impl DevVerifier {
    /// Construct the dev verifier, warning that it is not a real auth check.
    #[must_use]
    pub fn new() -> Self {
        tracing::warn!(
            "using DevVerifier: credentials are NOT authenticated — dev only, \
             replace with the persistence-backed verifier before production"
        );
        Self
    }
}

impl CredentialVerifier for DevVerifier {
    async fn verify(&self, creds: &Credentials) -> Result<AccountId, AuthError> {
        if creds.username.trim().is_empty() || creds.password.is_empty() {
            return Err(AuthError::InvalidCredentials);
        }
        // Stable, deterministic account id from the username — good enough to
        // exercise routing/session end-to-end in dev.
        Ok(AccountId::new(fnv1a(creds.username.as_bytes())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creds(u: &str, p: &str) -> Credentials {
        Credentials {
            username: u.to_string(),
            password: p.to_string(),
        }
    }

    #[tokio::test]
    async fn accepts_non_blank_credentials() {
        let v = DevVerifier;
        let account = v.verify(&creds("neo", "trinity")).await.unwrap();
        // Deterministic: same username → same account.
        let again = v.verify(&creds("neo", "different")).await.unwrap();
        assert_eq!(account, again);
    }

    #[tokio::test]
    async fn rejects_blank_username_or_password() {
        let v = DevVerifier;
        assert_eq!(
            v.verify(&creds("  ", "pw")).await,
            Err(AuthError::InvalidCredentials)
        );
        assert_eq!(
            v.verify(&creds("neo", "")).await,
            Err(AuthError::InvalidCredentials)
        );
    }
}
