//! Engine audio: spatial audio, clip playback, volume control, deterministic source
//! management.
//!
//! Audio playback is gated by the `audio` feature. In headless/server builds, the
//! audio system is unavailable; audio state (clips, sources) can still be inspected
//! via reflection for editor/replay tools.

use bevy_app::{Plugin, PluginGroup, PluginGroupBuilder};
use bevy_reflect::Reflect;

mod error;
pub use error::AudioError;

/// Audio engine plugin. Registers audio systems and assets when the `audio` feature
/// is enabled.
#[derive(Default)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, _app: &mut bevy_app::App) {
        #[cfg(feature = "audio")]
        {
            // Register audio types and systems here when bevy_audio support is added.
            _app.register_type::<AudioSourceInstance>();
        }
    }
}

/// Plugin group bundling all engine audio subsystems.
pub struct AudioPluginGroup;

impl PluginGroup for AudioPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(AudioPlugin)
    }
}

/// Marker component for audio source instances: position, volume, playback state.
/// Reflects state for editor inspection and replay.
#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct AudioSourceInstance {
    /// Volume: 0.0 (silent) to 1.0 (full).
    pub volume: f32,
    /// Whether playback is active.
    pub playing: bool,
}

#[cfg(test)]
mod tests {
    use bevy_app::App;

    use crate::AudioPlugin;

    #[test]
    fn plugin_builds() {
        let mut app = App::new();
        app.add_plugins(AudioPlugin);
        // Verify the plugin registers without panicking.
    }
}
