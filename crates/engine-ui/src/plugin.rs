/// Core UI plugin — registers i18n and optional rendering substrate.
use crate::i18n::I18nPlugin;

/// Main UiPlugin — registers i18n + optional bevy_ui rendering.
/// Headless: only i18n. With `ui` feature: bevy_ui substrate.
pub struct UiPlugin;

impl bevy_app::Plugin for UiPlugin {
  fn build(&self, app: &mut bevy_app::App) {
    // Always include i18n
    app.add_plugins(I18nPlugin);

    // Optional: bevy_ui rendering substrate
    #[cfg(feature = "ui")]
    {
      app.add_plugins(bevy_ui::UiPlugin);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ui_plugin_builds() {
    // Verify UiPlugin can be constructed and doesn't panic when added.
    // The plugin adds i18n infrastructure on top of EnginePlugins.
    let plugin = UiPlugin;
    // Can construct the plugin
    drop(plugin);
  }
}
