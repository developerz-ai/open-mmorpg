//! Engine audio: spatial mixing math, AoI-scoped emitter selection, and the
//! headful audio-device plugin.
//!
//! # Headless-first, device behind a feature
//! The math ([`spatial`]) and the relevance filter ([`aoi`]) are pure and
//! deterministic — no device, no ECS — so the server and replays can reason about
//! audibility with the same code the client mixes with. The actual audio device
//! (`bevy_audio`) is linked and added **only** under the non-default `audio`
//! feature, on the headful client path. CI builds that feature (it must compile),
//! but no test ever instantiates the device — the machine has no audio sink.
//!
//! → `docs/specs/game-engine/audio/README.md`.

pub mod aoi;
mod error;
pub mod spatial;

pub use aoi::{AudioAoi, EmitterInput, Voice};
pub use error::AudioError;
pub use spatial::{spatialize, Attenuation, Listener, Rolloff, SpatialMix};

use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

/// Audio engine plugin: registers audio types for the editor and, on the headful
/// client (`audio` feature), adds the `bevy_audio` device plugin.
///
/// Headless by default — safe to add to the server/sim app; it only touches a
/// device when the `audio` feature is enabled. Compose on top of
/// `omm_engine_core::EnginePlugins`.
#[derive(Debug, Default, Clone, Copy)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        register_audio_types(app);
        app.init_resource::<AudioSettings>();
        // The real audio device is headful-only. `bevy_audio::AudioPlugin` opens a
        // sink, which CI (no audio hardware) cannot do — so it is compiled in only
        // under the `audio` feature, and no test constructs this path.
        #[cfg(feature = "audio")]
        app.add_plugins(bevy_audio::AudioPlugin::default());
    }
}

/// Register every authored audio type with the reflection registry so the MCP
/// editor and agents can enumerate and author them. An unregistered type is
/// invisible to tooling — a bug.
pub fn register_audio_types(app: &mut App) {
    app.register_type::<AudioSettings>()
        .register_type::<AudioSourceInstance>()
        .register_type::<Attenuation>()
        .register_type::<Rolloff>()
        .register_type::<AudioAoi>();
}

/// Plugin group bundling all engine audio subsystems.
pub struct AudioPluginGroup;

impl PluginGroup for AudioPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(AudioPlugin)
    }
}

/// Global audio settings: master volume and the default emitter attenuation new
/// sources inherit. Reflected so the editor can tune the mix live.
#[derive(Debug, Clone, Copy, Resource, Reflect)]
#[reflect(Resource)]
pub struct AudioSettings {
    /// Master output gain, `0.0` (muted) .. `1.0` (unity).
    pub master_volume: f32,
    /// Default distance attenuation for emitters without an explicit one.
    pub default_attenuation: Attenuation,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            default_attenuation: Attenuation::default(),
        }
    }
}

/// Per-emitter playback state, reflected for editor inspection and replay.
#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct AudioSourceInstance {
    /// Volume: `0.0` (silent) to `1.0` (full).
    pub volume: f32,
    /// Whether playback is active.
    pub playing: bool,
}

#[cfg(test)]
mod tests {
    use core::any::TypeId;

    use bevy_app::App;
    use bevy_ecs::prelude::AppTypeRegistry;

    use super::*;

    /// Reflection registration must hold regardless of the `audio` feature, so
    /// this drives `register_audio_types` directly — never the device plugin.
    #[test]
    fn all_authored_types_are_registered() {
        let mut app = App::new();
        register_audio_types(&mut app);
        let registry = app.world().resource::<AppTypeRegistry>().read();
        for type_id in [
            TypeId::of::<AudioSettings>(),
            TypeId::of::<AudioSourceInstance>(),
            TypeId::of::<Attenuation>(),
            TypeId::of::<Rolloff>(),
            TypeId::of::<AudioAoi>(),
        ] {
            assert!(
                registry.get(type_id).is_some(),
                "unregistered audio type {type_id:?}"
            );
        }
    }

    /// Building the full plugin adds the `bevy_audio` device under the `audio`
    /// feature, which CI's device-enabled run cannot do — so only exercise the
    /// headless build here (feature off).
    #[cfg(not(feature = "audio"))]
    #[test]
    fn plugin_builds_headless_and_installs_settings() {
        let mut app = App::new();
        app.add_plugins(AudioPlugin);
        assert!(app.world().get_resource::<AudioSettings>().is_some());
    }
}
