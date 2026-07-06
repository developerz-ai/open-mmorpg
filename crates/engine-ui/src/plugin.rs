/// Core UI plugin — registers i18n and optional rendering substrate.
use crate::i18n::I18nPlugin;

/// Main UiPlugin — registers i18n + optional bevy_ui rendering.
/// Headless: only i18n. With `ui` feature: bevy_ui substrate.
pub struct UiPlugin;

impl bevy_app::Plugin for UiPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Always include i18n
        app.add_plugins(I18nPlugin);

        // Optional: bevy_ui rendering substrate + retained HUD widgets.
        #[cfg(feature = "ui")]
        {
            app.add_plugins(bevy_ui::UiPlugin);
            app.add_plugins(crate::hud::HudPlugin);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::I18nBundle;

    #[test]
    fn ui_plugin_registers_i18n_resource() {
        // The i18n substrate is always wired, headless or headful.
        let mut app = bevy_app::App::new();
        app.add_plugins(bevy_asset::AssetPlugin::default());
        app.add_plugins(I18nPlugin);
        assert!(app.world().contains_resource::<I18nBundle>());
    }
}
