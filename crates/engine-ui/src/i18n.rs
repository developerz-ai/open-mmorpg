//! Bevy glue for the i18n substrate: the `.ftl` asset type, the runtime
//! [`I18nBundle`] resource (compiled [`Catalog`] + [`LocaleFormatter`]), and the
//! [`I18nPlugin`]. The translation/formatting logic itself is pure and headless —
//! see [`crate::catalog`] and [`crate::format`].

use std::collections::HashMap;

use bevy_asset::{Asset, AssetApp, Handle};
use bevy_ecs::prelude::*;
use tracing::warn;

use crate::catalog::{Catalog, TransArgs};
use crate::error::I18nError;
use crate::format::LocaleFormatter;

/// A Fluent (`.ftl`) source file as a Bevy asset — enables file-watch hot-reload.
#[derive(Asset, Clone, Default, bevy_reflect::Reflect)]
pub struct I18nAsset {
    /// BCP-47 locale this source is for, e.g. `"en"`, `"fr-CA"`.
    pub locale: String,
    /// Raw FTL source text.
    pub source: String,
}

impl I18nAsset {
    /// Construct from a locale tag and FTL source.
    pub fn new(locale: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            locale: locale.into(),
            source: source.into(),
        }
    }
}

/// Runtime i18n state: the active locale, the compiled [`Catalog`], a matching
/// [`LocaleFormatter`], and `.ftl` asset handles per locale for hot-reload.
#[derive(Resource)]
pub struct I18nBundle {
    current_locale: String,
    catalog: Catalog,
    formatter: LocaleFormatter,
    handles: HashMap<String, Handle<I18nAsset>>,
}

impl Default for I18nBundle {
    fn default() -> Self {
        Self {
            current_locale: "en".to_owned(),
            catalog: Catalog::default(),
            formatter: LocaleFormatter::default(),
            handles: HashMap::new(),
        }
    }
}

impl I18nBundle {
    /// Build directly from `(locale, ftl)` sources — for tests and bootstrap.
    /// `default_locale` becomes the active locale and the formatter's locale.
    pub fn from_sources<'s>(
        default_locale: &str,
        sources: impl IntoIterator<Item = (&'s str, &'s str)>,
    ) -> Result<Self, I18nError> {
        let catalog = Catalog::from_sources(sources)?;
        Ok(Self {
            current_locale: default_locale.to_owned(),
            formatter: LocaleFormatter::new(default_locale),
            catalog,
            handles: HashMap::new(),
        })
    }

    /// The active locale.
    pub fn current_locale(&self) -> &str {
        &self.current_locale
    }

    /// Shared access to the compiled catalog.
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Mutable access to the catalog (e.g. to add/replace a bundle on hot-reload).
    pub fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }

    /// The formatter for the active locale.
    pub fn formatter(&self) -> &LocaleFormatter {
        &self.formatter
    }

    /// Translate `key` in the active locale. Missing → `⟦key⟧` (loud).
    pub fn t(&self, key: &str, args: &TransArgs) -> String {
        self.catalog.t(&self.current_locale, key, args)
    }

    /// Register a `.ftl` asset handle for a locale (hot-reload wiring).
    pub fn register(&mut self, locale: impl Into<String>, handle: Handle<I18nAsset>) {
        self.handles.insert(locale.into(), handle);
    }

    /// The asset handle registered for a locale, if any.
    pub fn handle(&self, locale: &str) -> Option<&Handle<I18nAsset>> {
        self.handles.get(locale)
    }

    /// Switch the active locale. Warns and no-ops if the catalog lacks it (so a
    /// typo can't silently drop the player into blank strings).
    pub fn set_locale(&mut self, locale: impl Into<String>) {
        let locale = locale.into();
        if self.catalog.supports(&locale) {
            self.formatter = LocaleFormatter::new(&locale);
            self.current_locale = locale;
        } else {
            warn!(locale = %locale, "i18n: locale not supported by catalog — keeping current");
        }
    }
}

/// Registers the `.ftl` asset type and the default [`I18nBundle`] resource.
/// Headless-safe: no rendering dependency.
pub struct I18nPlugin;

impl bevy_app::Plugin for I18nPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_asset::<I18nAsset>();
        app.init_resource::<I18nBundle>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: &str = "hello = Hello!\ngreet = Hi, { $name }!";
    const FR: &str = "hello = Bonjour !";

    fn bundle() -> I18nBundle {
        I18nBundle::from_sources("en", [("en", EN), ("fr", FR)]).expect("valid ftl")
    }

    #[test]
    fn defaults_to_en_empty_catalog() {
        let b = I18nBundle::default();
        assert_eq!(b.current_locale(), "en");
        assert_eq!(b.formatter().locale(), "en");
        // Empty catalog → every key is loud-missing, never blank.
        assert_eq!(b.t("hello", &TransArgs::new()), "⟦hello⟧");
    }

    #[test]
    fn translates_in_current_locale() {
        let b = bundle();
        assert_eq!(b.t("hello", &TransArgs::new()), "Hello!");
        assert_eq!(
            b.t("greet", &TransArgs::new().set("name", "Ada")),
            "Hi, Ada!"
        );
    }

    #[test]
    fn set_locale_switches_catalog_and_formatter() {
        let mut b = bundle();
        b.set_locale("fr");
        assert_eq!(b.current_locale(), "fr");
        assert_eq!(b.formatter().locale(), "fr");
        assert_eq!(b.t("hello", &TransArgs::new()), "Bonjour !");
    }

    #[test]
    fn set_locale_unsupported_is_noop_and_warns() {
        let mut b = bundle();
        b.set_locale("de");
        // Unsupported → stays on en; still resolves en strings.
        assert_eq!(b.current_locale(), "en");
        assert_eq!(b.t("hello", &TransArgs::new()), "Hello!");
    }

    #[test]
    fn set_locale_accepts_region_via_prefix() {
        let mut b = bundle();
        b.set_locale("fr-CA");
        assert_eq!(b.current_locale(), "fr-CA");
        // fr-CA has no bundle but falls back to fr.
        assert_eq!(b.t("hello", &TransArgs::new()), "Bonjour !");
    }

    #[test]
    fn plugin_builds_headless() {
        let mut app = bevy_app::App::new();
        app.add_plugins(bevy_asset::AssetPlugin::default());
        app.add_plugins(I18nPlugin);
        assert!(app.world().contains_resource::<I18nBundle>());
    }
}
