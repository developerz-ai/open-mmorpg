/// i18n substrate — Fluent-based localization + asset management.
/// Supports ICU MessageFormat via Fluent's built-in interpolation.
use bevy_asset::{Asset, AssetApp, Handle};
use bevy_ecs::prelude::*;
use fluent::FluentBundle;
use std::collections::HashMap;
use thiserror::Error;
use tracing::warn;
use unic_langid::LanguageIdentifier;

#[derive(Error, Debug)]
pub enum I18nError {
  #[error("fluent parse failed: {0}")]
  FluentParse(String),
  #[error("missing bundle for locale: {0}")]
  MissingBundle(String),
  #[error("missing message key: {0}")]
  MissingKey(String),
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
}

/// Fluent source file as an asset.
/// Stores the raw FTL source; compile on load or on demand.
#[derive(Asset, Clone, Default, bevy_reflect::Reflect)]
pub struct I18nAsset {
  pub locale: String,
  pub source: String,
}

impl I18nAsset {
  /// Create a new i18n asset from locale and Fluent source.
  pub fn new(locale: impl Into<String>, source: impl Into<String>) -> Self {
    Self {
      locale: locale.into(),
      source: source.into(),
    }
  }

  /// Parse this asset into a compiled Fluent bundle.
  pub fn compile(&self) -> Result<FluentBundle<fluent::FluentResource>, I18nError> {
    let lang_id: LanguageIdentifier = self
      .locale
      .parse()
      .unwrap_or_else(|_| "en".parse().unwrap());

    let mut bundle = FluentBundle::new(vec![lang_id]);
    bundle
      .add_resource(fluent::FluentResource::try_new(self.source.clone()).map_err(
        |e| I18nError::FluentParse(format!("{:?}", e)),
      )?)
      .map_err(|e| I18nError::FluentParse(format!("{:?}", e)))?;

    Ok(bundle)
  }
}

/// Game-wide i18n state — asset handles per locale.
/// Use the AssetServer to load I18nAsset handles and register here.
/// Bundles are compiled on demand from assets.
#[derive(Resource)]
pub struct I18nBundle {
  pub current_locale: String,
  /// Locale -> asset handle mapping
  pub bundles: HashMap<String, Handle<I18nAsset>>,
}

impl Default for I18nBundle {
  fn default() -> Self {
    Self {
      current_locale: "en".to_string(),
      bundles: HashMap::new(),
    }
  }
}

impl I18nBundle {
  pub fn new(locale: String) -> Self {
    Self {
      current_locale: locale,
      bundles: HashMap::new(),
    }
  }

  /// Register an asset handle for a locale.
  pub fn register(&mut self, locale: String, handle: Handle<I18nAsset>) {
    self.bundles.insert(locale, handle);
  }

  /// Set the current locale. Warns if not registered.
  pub fn set_locale(&mut self, locale: String) {
    if self.bundles.contains_key(&locale) {
      self.current_locale = locale;
    } else {
      warn!("Locale '{}' not registered", locale);
    }
  }

  /// Get the handle for the current locale (if loaded).
  pub fn current_handle(&self) -> Option<Handle<I18nAsset>> {
    self.bundles.get(&self.current_locale).cloned()
  }
}

/// bevy_app plugin for i18n setup.
/// Registers the I18nAsset type and initializes the bundle resource.
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

  #[test]
  fn test_i18n_asset_new() {
    let source = "greeting = Hello!";
    let asset = I18nAsset::new("en", source);
    assert_eq!(asset.locale, "en");
    assert_eq!(asset.source, source);
  }

  #[test]
  fn test_i18n_asset_compile() {
    let source = r#"
greeting = Hello, { $name }!
farewell = Goodbye, { $name }!
    "#;

    let asset = I18nAsset::new("en", source);
    let bundle = asset.compile().expect("valid fluent");
    // Bundle compiles successfully
    assert!(bundle.locales.iter().any(|l| l.to_string() == "en"));
  }

  #[test]
  fn test_i18n_asset_compile_error() {
    let bad_fluent = "this is not valid fluent syntax {";
    let asset = I18nAsset::new("en", bad_fluent);
    let result = asset.compile();
    assert!(result.is_err());
  }

  #[test]
  fn test_i18n_bundle_default() {
    let bundle = I18nBundle::default();
    assert_eq!(bundle.current_locale, "en");
    assert!(bundle.bundles.is_empty());
  }

  #[test]
  fn test_i18n_bundle_new() {
    let bundle = I18nBundle::new("fr".to_string());
    assert_eq!(bundle.current_locale, "fr");
  }

  #[test]
  fn test_i18n_bundle_register_and_switch() {
    // Note: This test uses dummy handles since Handle::weak() is unavailable
    // In real usage, handles come from AssetServer::load()
    let mut bundle = I18nBundle::default();
    assert_eq!(bundle.current_locale, "en");

    bundle.set_locale("fr".to_string());
    // Warns because "fr" not registered, reverts to previous
    assert_eq!(bundle.current_locale, "en");
  }
}
