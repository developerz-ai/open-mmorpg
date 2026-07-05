//! Game client — **Linux-first** (other targets come later; the goal says don't
//! worry about them yet).
//!
//! The renderer will be Bevy (wgpu + PBR), but rendering pulls in platform
//! graphics/audio system libraries, so the scaffold ships the part that is pure
//! and CI-friendly: the **client-side prediction core**. The client runs the
//! *same* `omm-sim` the server runs, so it can predict locally from intents and
//! later reconcile against authoritative snapshots. When the renderer lands it
//! draws whatever this core predicts (docs/architecture/03-netcode-and-sharding.md).
//!
//! The client is built to run **headless or headful from one core**: headless
//! (this binary) is deterministic and drivable by tests and AI agents — connect
//! an agent, feed intents, assert predicted state — while the headful build adds
//! the renderer on top of the *same* prediction core. Great for testing, great
//! for agentic play (see `apps/mcp` companions).
//!
//! Operators may ship their own client build — extra maps, modified rules — on
//! top of this same prediction core. Core stays strong; the surface is theirs.

use omm_protocol::{Intent, Vec3};
use omm_sim::{simulate, EntityState};

/// Predict local state from a sequence of the player's own intents, starting
/// from the last acknowledged authoritative state.
fn predict(from: EntityState, local_inputs: &[Intent]) -> EntityState {
    simulate(from, local_inputs)
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    // Demonstrate the prediction core: push forward for a few ticks.
    let inputs = vec![
        Intent::Move {
            dir: Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0
            }
        };
        30
    ];
    let predicted = predict(EntityState::default(), &inputs);
    tracing::info!(
        "predicted position after {} ticks: x={:.3}",
        inputs.len(),
        predicted.pos.0.x
    );
    println!(
        "omm-client (headless prediction core) — predicted x={:.3}",
        predicted.pos.0.x
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prediction_matches_the_shared_sim() {
        let inputs = vec![
            Intent::Move {
                dir: Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0
                }
            };
            30
        ];
        let predicted = predict(EntityState::default(), &inputs);
        // 30 ticks * (1.0 * 1/30) = 1.0 unit.
        assert!((predicted.pos.0.x - 1.0).abs() < 1e-4);
    }
}
