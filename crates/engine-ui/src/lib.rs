#![doc = include_str!("../README.md")]

mod catalog;
mod error;
mod format;
mod i18n;
mod inspector;
mod plugin;

#[cfg(feature = "ui")]
mod hud;
#[cfg(feature = "ui")]
mod ui;

pub use catalog::{missing_marker, Catalog, TransArgs};
pub use error::I18nError;
pub use format::{Currency, LocaleFormatter};
pub use i18n::{I18nAsset, I18nBundle, I18nPlugin};
pub use inspector::{
    describe_by_path, describe_registration, describe_type, inspect_components, WidgetDescriptor,
    WidgetKind, MAX_DEPTH,
};
pub use plugin::UiPlugin;

#[cfg(feature = "ui")]
pub use hud::{
    spawn_health_bar, spawn_nameplate, sync_health_bars, sync_nameplates, HealthBar, HudPlugin,
    Nameplate,
};
#[cfg(feature = "ui")]
pub use ui::*;
