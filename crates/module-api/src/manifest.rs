//! Module identity — the compile-time facts a module states about itself.
//!
//! The build-time manifest a fork authors lives in `modules/<name>/module.toml`
//! (name, version, `core-api-version`, declared hooks) for humans and the
//! scaffold to read. [`ModuleManifest`] is its runtime mirror: what a loaded
//! module reports for logging and operator inspection.

/// The hook-API version this build of `omm-module-api` exposes.
///
/// Bumped on a breaking change to any hook signature or context type. A module
/// records the version it was authored against in its `module.toml`; because
/// modules compile *in the same workspace* as the core, an incompatible module
/// fails at compile time — this constant is the human-readable contract, not a
/// runtime gate that could silently admit a stale plugin (the `.so` failure mode
/// we avoid by not shipping `.so`s at all).
pub const CORE_API_VERSION: u32 = 1;

/// A loaded module's identity, reported at runtime for logs and tooling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleManifest {
    /// Stable module name, matching its `modules/<name>/` directory.
    pub name: &'static str,
    /// The module's own semver, from its `CARGO_PKG_VERSION`.
    pub version: &'static str,
    /// The [`CORE_API_VERSION`] the module was built against.
    pub core_api_version: u32,
}

impl ModuleManifest {
    /// A manifest for a module built against the current [`CORE_API_VERSION`].
    ///
    /// Const so a module can declare it without a runtime cost. Pass
    /// `env!("CARGO_PKG_VERSION")` for `version` so identity tracks the crate.
    #[must_use]
    pub const fn new(name: &'static str, version: &'static str) -> Self {
        Self {
            name,
            version,
            core_api_version: CORE_API_VERSION,
        }
    }

    /// Whether the module's API version matches the core it is linked into.
    ///
    /// Always true for a module compiled in this workspace; exposed so operator
    /// tooling can surface the version it was built against explicitly.
    #[must_use]
    pub const fn is_compatible(&self) -> bool {
        self.core_api_version == CORE_API_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stamps_the_current_api_version() {
        let m = ModuleManifest::new("demo", "1.2.3");
        assert_eq!(m.name, "demo");
        assert_eq!(m.version, "1.2.3");
        assert_eq!(m.core_api_version, CORE_API_VERSION);
        assert!(m.is_compatible());
    }

    #[test]
    fn a_mismatched_version_is_incompatible() {
        let m = ModuleManifest {
            name: "stale",
            version: "0.0.1",
            core_api_version: CORE_API_VERSION + 1,
        };
        assert!(!m.is_compatible());
    }
}
