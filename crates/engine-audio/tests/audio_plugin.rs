//! Integration tests for the audio plugin scaffold.

#[test]
fn audio_plugin_loads() {
    // Verify that the crate can be imported and compiled without errors.
    use omm_engine_audio::AudioPlugin;

    // Plugin is constructible and has the expected interface.
    let _plugin = AudioPlugin::default();
}
