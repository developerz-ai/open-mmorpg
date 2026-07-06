#![cfg(feature = "ui")]

//! bevy_ui rendering substrate — available only with `ui` feature.
//! Headless / server / tests: not compiled.

use bevy_ecs::component::Component;

/// Marker component for UI elements.
#[derive(Component)]
pub struct UiElement;

/// Placeholder for future UI layout/styling helpers.
#[allow(dead_code)]
pub struct UiBuilder;

impl UiBuilder {
  pub fn new() -> Self {
    Self
  }
}

impl Default for UiBuilder {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ui_element_marker() {
    let _elem = UiElement;
    // Component can be instantiated
  }

  #[test]
  fn test_ui_builder() {
    let _builder = UiBuilder::new();
    let _builder = UiBuilder::default();
  }
}
