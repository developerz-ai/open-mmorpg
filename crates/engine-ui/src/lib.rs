#![doc = include_str!("../README.md")]

mod i18n;
mod plugin;

#[cfg(feature = "ui")]
mod ui;

pub use i18n::{I18nAsset, I18nBundle, I18nPlugin};
pub use plugin::UiPlugin;

#[cfg(feature = "ui")]
pub use ui::*;
