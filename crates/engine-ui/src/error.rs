//! Typed errors for the i18n substrate.

use thiserror::Error;

/// Errors raised while building an i18n [`Catalog`](crate::Catalog).
///
/// Note: a *missing key at lookup time is not an error* — it renders loudly as
/// `⟦key⟧` (see [`crate::missing_marker`]). Errors here are authoring faults:
/// malformed `.ftl` sources caught while compiling the catalog.
#[derive(Error, Debug)]
pub enum I18nError {
    /// A Fluent (`.ftl`) source failed to parse.
    #[error("fluent parse failed for locale `{locale}`: {detail}")]
    FluentParse {
        /// The locale whose source failed to parse.
        locale: String,
        /// Debug rendering of the underlying parser errors.
        detail: String,
    },
    /// A parsed resource could not be added to the bundle (e.g. duplicate id).
    #[error("fluent resource add failed for locale `{locale}`: {detail}")]
    ResourceAdd {
        /// The locale whose resource failed to add.
        locale: String,
        /// Debug rendering of the underlying bundle errors.
        detail: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_is_actionable() {
        let e = I18nError::FluentParse {
            locale: "pl".into(),
            detail: "boom".into(),
        };
        assert!(e.to_string().contains("pl"));
        assert!(e.to_string().contains("boom"));
    }
}
