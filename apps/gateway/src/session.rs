//! Signed session tokens.
//!
//! On successful login the gateway issues a short-lived, HMAC-signed token that
//! shards validate on connect (`docs/specs/game-server/security/README.md`:
//! "short-lived signed session tokens issued by gateway, validated per shard").
//!
//! Wire format: `base64url(payload).base64url(HMAC-SHA256(payload, secret))`.
//! The payload is a fixed 32-byte, big-endian encoding of the [`Claims`] — a
//! compact, allocation-free, infallible codec (no serde on the token path). The
//! signing secret comes from config/env and is **never** logged.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use hmac::{Hmac, Mac};
use omm_protocol::AccountId;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq as _;

use crate::error::AuthError;

type HmacSha256 = Hmac<Sha256>;

/// Seconds since the Unix epoch — the clock unit for token issue/expiry.
///
/// Wall-clock time is confined to the control plane; it never touches the
/// deterministic sim. Kept as a newtype so an expiry can't be swapped for an
/// issue time by accident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixSeconds(pub u64);

impl UnixSeconds {
    /// The current wall-clock time. Falls back to `0` if the system clock is
    /// before the Unix epoch (which only fails a still-valid token, never
    /// forges one).
    #[must_use]
    pub fn now() -> Self {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self(secs)
    }
}

/// The verified contents of a session token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Claims {
    /// The authenticated account this token authorizes.
    pub account: AccountId,
    /// When the token was issued.
    pub issued_at: UnixSeconds,
    /// When the token expires (exclusive) — rejected once `now >= expires_at`.
    pub expires_at: UnixSeconds,
    /// Random per-token nonce, so two tokens for the same account differ and a
    /// leaked one can be individually distinguished/revoked in logs.
    pub nonce: u64,
}

/// An opaque, signed session token. Its `String` form is what the client holds
/// and later presents to a shard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token(String);

impl Token {
    /// Wrap an existing token string (e.g. one arriving from a client).
    #[must_use]
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// Consume into the owned string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

/// Issues and verifies session tokens with a single HMAC secret.
///
/// Holds the raw secret; `Debug` is deliberately opaque so a secret can never
/// leak into a log line.
pub struct SessionSigner {
    secret: Vec<u8>,
}

impl std::fmt::Debug for SessionSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionSigner")
            .field("secret", &"<redacted>")
            .finish()
    }
}

const PAYLOAD_LEN: usize = 32;

impl SessionSigner {
    /// Build a signer from raw secret bytes.
    #[must_use]
    pub fn new(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Issue a token for `account`, valid for `ttl_secs` from `now`.
    ///
    /// # Errors
    /// Returns [`AuthError::Internal`] only if the HMAC key is unusable, which
    /// cannot happen for `Hmac` (it accepts any key length).
    pub fn issue(
        &self,
        account: AccountId,
        now: UnixSeconds,
        ttl_secs: u64,
    ) -> Result<Token, AuthError> {
        let claims = Claims {
            account,
            issued_at: now,
            expires_at: UnixSeconds(now.0.saturating_add(ttl_secs)),
            nonce: rand::random(),
        };
        let payload = encode_payload(&claims);
        let mac = self.mac(&payload)?;
        let token = format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(payload),
            URL_SAFE_NO_PAD.encode(mac)
        );
        Ok(Token(token))
    }

    /// Verify a token against `now`, returning its [`Claims`] on success.
    ///
    /// # Errors
    /// - [`AuthError::Malformed`] — not `payload.mac`, or bad base64/length.
    /// - [`AuthError::BadSignature`] — MAC mismatch (tampering).
    /// - [`AuthError::Expired`] — `now >= expires_at`.
    pub fn verify(&self, token: &Token, now: UnixSeconds) -> Result<Claims, AuthError> {
        let (payload_b64, mac_b64) = token.0.split_once('.').ok_or(AuthError::Malformed)?;
        let payload = URL_SAFE_NO_PAD
            .decode(payload_b64)
            .map_err(|_| AuthError::Malformed)?;
        let presented = URL_SAFE_NO_PAD
            .decode(mac_b64)
            .map_err(|_| AuthError::Malformed)?;

        let expected = self.mac(&payload)?;
        // Length check first, then constant-time compare of equal-length slices:
        // a mismatched length is a structural (non-secret) reject, not a timing
        // leak of the secret MAC.
        if presented.len() != expected.len() || !bool::from(presented.ct_eq(&expected)) {
            return Err(AuthError::BadSignature);
        }

        let claims = decode_payload(&payload)?;
        if now.0 >= claims.expires_at.0 {
            return Err(AuthError::Expired);
        }
        Ok(claims)
    }

    /// Compute HMAC-SHA256 of `data`. The `InvalidLength` branch is unreachable
    /// for `Hmac` (any key length is valid); it maps to `Internal` defensively.
    fn mac(&self, data: &[u8]) -> Result<[u8; 32], AuthError> {
        let mut mac = HmacSha256::new_from_slice(&self.secret).map_err(|_| AuthError::Internal)?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().into())
    }
}

/// Encode claims to the fixed 32-byte big-endian payload.
fn encode_payload(c: &Claims) -> [u8; PAYLOAD_LEN] {
    let mut buf = [0u8; PAYLOAD_LEN];
    buf[0..8].copy_from_slice(&c.account.raw().to_be_bytes());
    buf[8..16].copy_from_slice(&c.issued_at.0.to_be_bytes());
    buf[16..24].copy_from_slice(&c.expires_at.0.to_be_bytes());
    buf[24..32].copy_from_slice(&c.nonce.to_be_bytes());
    buf
}

/// Decode the fixed payload back into claims, rejecting any wrong length.
fn decode_payload(bytes: &[u8]) -> Result<Claims, AuthError> {
    if bytes.len() != PAYLOAD_LEN {
        return Err(AuthError::Malformed);
    }
    let read = |lo: usize| -> Result<u64, AuthError> {
        let slice: [u8; 8] = bytes[lo..lo + 8]
            .try_into()
            .map_err(|_| AuthError::Malformed)?;
        Ok(u64::from_be_bytes(slice))
    };
    Ok(Claims {
        account: AccountId::new(read(0)?),
        issued_at: UnixSeconds(read(8)?),
        expires_at: UnixSeconds(read(16)?),
        nonce: read(24)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signer() -> SessionSigner {
        SessionSigner::new(b"test-secret-key".to_vec())
    }

    #[test]
    fn issue_then_verify_roundtrips() {
        let s = signer();
        let now = UnixSeconds(1_000);
        let token = s.issue(AccountId::new(42), now, 60).unwrap();
        let claims = s.verify(&token, UnixSeconds(1_030)).unwrap();
        assert_eq!(claims.account, AccountId::new(42));
        assert_eq!(claims.issued_at, now);
        assert_eq!(claims.expires_at, UnixSeconds(1_060));
    }

    #[test]
    fn tampered_payload_is_rejected() {
        let s = signer();
        let token = s.issue(AccountId::new(7), UnixSeconds(0), 60).unwrap();
        // Decode the payload, flip a byte, then re-encode: a well-formed,
        // correct-length payload whose contents differ — so it fails the HMAC
        // check (BadSignature) rather than base64 decoding (Malformed). Flipping
        // a raw base64 char instead would be flaky: mangling the final char can
        // set non-canonical trailing bits and be rejected as Malformed.
        let raw = token.into_string();
        let (payload_b64, mac) = raw.split_once('.').unwrap();
        let mut payload = URL_SAFE_NO_PAD.decode(payload_b64).unwrap();
        payload[0] ^= 0x01;
        let forged = Token::new(format!("{}.{mac}", URL_SAFE_NO_PAD.encode(&payload)));
        assert_eq!(
            s.verify(&forged, UnixSeconds(1)),
            Err(AuthError::BadSignature)
        );
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let good = signer();
        let evil = SessionSigner::new(b"other-secret".to_vec());
        let token = good.issue(AccountId::new(1), UnixSeconds(0), 60).unwrap();
        assert_eq!(
            evil.verify(&token, UnixSeconds(1)),
            Err(AuthError::BadSignature)
        );
    }

    #[test]
    fn expired_token_is_rejected() {
        let s = signer();
        let token = s.issue(AccountId::new(9), UnixSeconds(100), 10).unwrap();
        // now == expires_at (exclusive) → expired.
        assert_eq!(s.verify(&token, UnixSeconds(110)), Err(AuthError::Expired));
        assert_eq!(s.verify(&token, UnixSeconds(200)), Err(AuthError::Expired));
    }

    #[test]
    fn malformed_token_is_rejected() {
        let s = signer();
        assert_eq!(
            s.verify(&Token::new("no-dot"), UnixSeconds(0)),
            Err(AuthError::Malformed)
        );
        assert_eq!(
            s.verify(&Token::new("!!!.???"), UnixSeconds(0)),
            Err(AuthError::Malformed)
        );
        // Valid base64 but wrong payload length.
        let short = URL_SAFE_NO_PAD.encode([0u8; 8]);
        let token = Token::new(format!("{short}.{short}"));
        assert!(matches!(
            s.verify(&token, UnixSeconds(0)),
            Err(AuthError::BadSignature | AuthError::Malformed)
        ));
    }

    #[test]
    fn nonce_differs_per_issue() {
        let s = signer();
        let a = s.issue(AccountId::new(5), UnixSeconds(0), 60).unwrap();
        let b = s.issue(AccountId::new(5), UnixSeconds(0), 60).unwrap();
        assert_ne!(a, b, "random nonce must make tokens unique");
    }

    #[test]
    fn debug_does_not_leak_secret() {
        let dbg = format!("{:?}", SessionSigner::new(b"super-secret".to_vec()));
        assert!(!dbg.contains("super-secret"));
        assert!(dbg.contains("redacted"));
    }
}
