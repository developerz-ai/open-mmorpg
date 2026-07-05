//! Typed, secret-safe database configuration.
//!
//! The connection URL carries the password, so it is wrapped in a type whose
//! `Debug` redacts — a stray `{:?}` can never leak credentials into a log line.

use omm_errors::{CoreError, CoreResult};
use std::time::Duration;

/// A connection URL that never prints its contents.
#[derive(Clone)]
struct SecretUrl(String);

impl std::fmt::Debug for SecretUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<redacted>")
    }
}

/// How to reach and pool the durable store (YugabyteDB — Postgres wire).
#[derive(Clone, Debug)]
pub struct DbConfig {
    url: SecretUrl,
    /// Upper bound on pooled connections.
    pub max_connections: u32,
    /// How long a single connect attempt may run before it is retried.
    pub connect_timeout: Duration,
    /// How long `acquire` waits for a free pooled connection.
    pub acquire_timeout: Duration,
    /// How many times [`connect`](crate::connect) retries before giving up.
    pub connect_retries: u32,
}

impl DbConfig {
    /// Configuration with sane defaults around a connection `url`.
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: SecretUrl(url.into()),
            max_connections: 10,
            connect_timeout: Duration::from_secs(10),
            acquire_timeout: Duration::from_secs(30),
            connect_retries: 5,
        }
    }

    /// Read configuration from the environment. `DATABASE_URL` is required;
    /// `DB_MAX_CONNECTIONS` optionally overrides the pool size.
    ///
    /// # Errors
    /// [`CoreError::BadRequest`] if `DATABASE_URL` is unset or an override does
    /// not parse.
    pub fn from_env() -> CoreResult<Self> {
        let url = std::env::var("DATABASE_URL")
            .map_err(|_| CoreError::BadRequest("DATABASE_URL is not set".into()))?;
        let mut cfg = Self::new(url);
        if let Ok(raw) = std::env::var("DB_MAX_CONNECTIONS") {
            cfg.max_connections = raw
                .parse()
                .map_err(|_| CoreError::BadRequest("DB_MAX_CONNECTIONS must be a u32".into()))?;
        }
        Ok(cfg)
    }

    /// The raw URL — for the pool builder only, never for logging.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_leaks_the_password() {
        let cfg = DbConfig::new("postgres://user:hunter2@db:5433/realm");
        let rendered = format!("{cfg:?}");
        assert!(!rendered.contains("hunter2"), "password leaked: {rendered}");
        assert!(rendered.contains("<redacted>"));
    }

    #[test]
    fn new_applies_sane_defaults() {
        let cfg = DbConfig::new("postgres://localhost/realm");
        assert_eq!(cfg.max_connections, 10);
        assert_eq!(cfg.connect_retries, 5);
        assert_eq!(cfg.url(), "postgres://localhost/realm");
    }
}
