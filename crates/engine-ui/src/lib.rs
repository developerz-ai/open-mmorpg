#![doc = include_str!("../README.md")]

mod catalog;
mod error;
mod format;
mod i18n;
mod plugin;

#[cfg(feature = "ui")]
mod ui;

pub use catalog::{missing_marker, Catalog, TransArgs};
pub use error::I18nError;
pub use format::{Currency, LocaleFormatter};
pub use i18n::{I18nAsset, I18nBundle, I18nPlugin};
pub use plugin::UiPlugin;

#[cfg(feature = "ui")]
pub use ui::*;
