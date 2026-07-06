#![cfg(feature = "ui")]

//! bevy_ui rendering substrate — available only with `ui` feature.
//! Headless / server / tests: not compiled.

use bevy_ecs::component::Component;

/// Marker component for UI elements.
#[derive(Component)]
pub struct UiElement;

/// Placeholder for future UI layout/styling helpers.
#[derive(Default)]
#[allow(dead_code)]
pub struct UiBuilder;

impl UiBuilder {
    /// Construct a new UI builder.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_element_marker_constructs() {
        let _elem = UiElement;
    }

    #[test]
    fn ui_builder_constructs() {
        let _a = UiBuilder::new();
        let _b = UiBuilder;
    }
}
