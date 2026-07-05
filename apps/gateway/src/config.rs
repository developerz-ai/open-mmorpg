//! Gateway configuration, loaded from the environment with dev defaults.
//!
//! The signing secret comes from `GATEWAY_SECRET`; a dev default is used if it
//! is unset, but only with a loud warning — a real secret is never hardcoded
//! into a shipped default (`docs/specs/game-server/security/README.md`).

use omm_protocol::ShardId;
use std::net::SocketAddr;

/// A dev-only signing secret. Safe precisely because it is public and obviously
/// not a real secret; production must set `GATEWAY_SECRET`.
const DEV_SECRET: &str = "dev-insecure-gateway-secret-do-not-ship";

/// Resolved gateway settings.
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Address the HTTP control plane binds to.
    pub bind: SocketAddr,
    /// HMAC secret for session tokens (raw bytes).
    pub secret: Vec<u8>,
    /// The shard set zones are routed across.
    pub shards: Vec<ShardId>,
    /// Session token lifetime in seconds.
    pub token_ttl_secs: u64,
    /// Public realm display name.
    pub realm_name: String,
    /// Advertised realm capacity (soft — no hard queue).
    pub realm_capacity: u32,
    /// Login rate-limit burst size.
    pub login_burst: u32,
    /// Login rate-limit sustained rate (requests/second).
    pub login_refill_per_sec: f64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            bind: SocketAddr::from(([0, 0, 0, 0], 8080)),
            secret: DEV_SECRET.as_bytes().to_vec(),
            shards: vec![ShardId::new(1), ShardId::new(2), ShardId::new(3)],
            token_ttl_secs: 3600,
            realm_name: "open-mmorpg".to_string(),
            realm_capacity: 100_000,
            login_burst: 5,
            login_refill_per_sec: 1.0,
        }
    }
}

impl GatewayConfig {
    /// Build config from environment variables, falling back to [`Default`].
    ///
    /// Recognized vars: `GATEWAY_BIND`, `GATEWAY_SECRET`, `GATEWAY_SHARDS`
    /// (comma-separated ids), `GATEWAY_TOKEN_TTL_SECS`. Unset or unparseable
    /// values keep the default. A missing/blank secret logs a warning and uses
    /// the insecure dev secret.
    #[must_use]
    pub fn from_env() -> Self {
        let mut cfg = Self::default();

        if let Some(bind) = std::env::var("GATEWAY_BIND")
            .ok()
            .and_then(|s| s.parse().ok())
        {
            cfg.bind = bind;
        }

        match std::env::var("GATEWAY_SECRET") {
            Ok(s) if !s.trim().is_empty() => cfg.secret = s.into_bytes(),
            _ => tracing::warn!(
                "GATEWAY_SECRET unset — using an INSECURE dev signing secret; \
                 set GATEWAY_SECRET in any real deployment"
            ),
        }

        if let Ok(list) = std::env::var("GATEWAY_SHARDS") {
            let shards = parse_shards(&list);
            if !shards.is_empty() {
                cfg.shards = shards;
            }
        }

        if let Some(ttl) = std::env::var("GATEWAY_TOKEN_TTL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
        {
            cfg.token_ttl_secs = ttl;
        }

        cfg
    }
}

/// Parse a comma-separated list of shard ids, silently skipping bad entries.
fn parse_shards(list: &str) -> Vec<ShardId> {
    list.split(',')
        .filter_map(|s| s.trim().parse::<u64>().ok())
        .map(ShardId::new)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_usable() {
        let cfg = GatewayConfig::default();
        assert!(!cfg.shards.is_empty());
        assert!(!cfg.secret.is_empty());
        assert_eq!(cfg.token_ttl_secs, 3600);
    }

    #[test]
    fn parses_shard_list_skipping_junk() {
        assert_eq!(
            parse_shards("1, 2 ,x, 3"),
            vec![ShardId::new(1), ShardId::new(2), ShardId::new(3)]
        );
        assert!(parse_shards("").is_empty());
    }
}
