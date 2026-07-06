#![cfg(feature = "ui")]

//! Retained HUD widgets — persistent, ECS-native `bevy_ui` game UI (nameplates,
//! health bars). Available only with the `ui` feature; the headless core and
//! server never link it (→ [UI spec](../../../docs/specs/game-engine/ui/README.md)).
//!
//! "Retained" means the widgets are long-lived entities: spawn once, then a
//! per-frame system binds their visual state to gameplay data — a health bar's
//! node *width* tracks the (clamped) health fraction; a nameplate's *text* is
//! resolved through i18n. No immediate-mode rebuild each frame (that mode is for
//! tools/editor, not game UI).
//!
//! Both widget types are `#[reflect(Component)]`, so [`crate::inspector`] can
//! enumerate and describe them like any other component — the same reflection
//! that feeds the editor and MCP.

use bevy_app::{App, Plugin, Update};
use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use bevy_ui::widget::Text;
use bevy_ui::{BackgroundColor, Node, Val};

use crate::catalog::TransArgs;
use crate::i18n::I18nBundle;

/// A retained health-bar widget's data. The bar node's width is kept at
/// [`HealthBar::fraction`] of its track by [`sync_health_bars`].
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct HealthBar {
    /// Current health. Clamped into `[0, max]` when computing the fill.
    pub current: f32,
    /// Maximum health. A non-positive max renders an empty bar (never a NaN).
    pub max: f32,
}

impl Default for HealthBar {
    fn default() -> Self {
        Self {
            current: 1.0,
            max: 1.0,
        }
    }
}

impl HealthBar {
    /// A health bar with the given current/max values.
    #[must_use]
    pub fn new(current: f32, max: f32) -> Self {
        Self { current, max }
    }

    /// Fill fraction in `[0, 1]`. Robust against bad data: a non-positive or
    /// non-finite `max`, or a non-finite `current`, yields `0.0` rather than a
    /// NaN width that would corrupt layout. Over-heal (`current > max`) clamps
    /// to a full bar.
    #[must_use]
    pub fn fraction(&self) -> f32 {
        if self.max <= 0.0 || !self.max.is_finite() || !self.current.is_finite() {
            return 0.0;
        }
        (self.current / self.max).clamp(0.0, 1.0)
    }

    /// The fill width as a percentage [`Val`] for the bar's node.
    #[must_use]
    pub fn fill(&self) -> Val {
        Val::Percent(self.fraction() * 100.0)
    }
}

/// A retained nameplate widget's data: a verbatim display name plus an optional
/// translated subtitle line.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Default)]
#[reflect(Component)]
pub struct Nameplate {
    /// Display name — dynamic data (player/NPC name), shown **verbatim**, never
    /// routed through the string catalog.
    pub name: String,
    /// Optional i18n key for a subtitle/rank line, resolved via `t()`. A missing
    /// key renders loudly (`⟦key⟧`), never a silent blank.
    pub subtitle_key: Option<String>,
}

impl Nameplate {
    /// A nameplate showing `name` with no subtitle.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            subtitle_key: None,
        }
    }

    /// Attach a translated subtitle line, addressed by i18n key.
    #[must_use]
    pub fn with_subtitle(mut self, key: impl Into<String>) -> Self {
        self.subtitle_key = Some(key.into());
        self
    }

    /// Resolve the display string: the verbatim name, plus the translated
    /// subtitle on a second line when a key is set.
    #[must_use]
    pub fn resolve(&self, i18n: &I18nBundle) -> String {
        match &self.subtitle_key {
            Some(key) => format!("{}\n{}", self.name, i18n.t(key, &TransArgs::new())),
            None => self.name.clone(),
        }
    }
}

/// Spawn a retained health-bar widget: a `bevy_ui` node coloured `fill` whose
/// width already reflects `bar`. Returns the entity so the caller can parent it
/// under a track/background and position it; [`sync_health_bars`] keeps the
/// width current as the `HealthBar` changes.
pub fn spawn_health_bar(commands: &mut Commands, bar: HealthBar, fill: Color) -> Entity {
    commands
        .spawn((
            Node {
                width: bar.fill(),
                height: Val::Percent(100.0),
                ..Node::default()
            },
            BackgroundColor(fill),
            bar,
        ))
        .id()
}

/// Spawn a retained nameplate widget. Its `Text` starts at the verbatim name;
/// [`sync_nameplates`] fills in the translated subtitle on the next frame.
pub fn spawn_nameplate(commands: &mut Commands, nameplate: Nameplate) -> Entity {
    let text = Text(nameplate.name.clone());
    commands.spawn((text, nameplate)).id()
}

/// Bind each changed health bar's node width to its clamped fill fraction.
/// `Changed` filter: only touched bars re-layout, so idle HUDs cost nothing.
pub fn sync_health_bars(mut bars: Query<(&HealthBar, &mut Node), Changed<HealthBar>>) {
    for (bar, mut node) in &mut bars {
        node.width = bar.fill();
    }
}

/// Resolve nameplate text through i18n, writing only on a real change so a
/// locale switch updates every plate while steady state stays quiet.
pub fn sync_nameplates(i18n: Res<I18nBundle>, mut plates: Query<(&Nameplate, &mut Text)>) {
    for (plate, mut text) in &mut plates {
        let resolved = plate.resolve(&i18n);
        if text.0 != resolved {
            text.0 = resolved;
        }
    }
}

/// Registers the retained HUD widget types (for reflection) and their per-frame
/// sync systems. Headful only; added by [`crate::UiPlugin`] under the `ui`
/// feature. Expects an [`I18nBundle`] resource (installed by the i18n plugin).
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HealthBar>()
            .register_type::<Nameplate>()
            .add_systems(Update, (sync_health_bars, sync_nameplates));
    }
}

#[cfg(test)]
mod tests;
