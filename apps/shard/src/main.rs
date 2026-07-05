//! Shard — a headless authoritative zone server.
//!
//! It owns one slice of the world: it advances the deterministic simulation on a
//! fixed tick, applies validated client intents, and streams snapshots over the
//! netcode transport. Deterministic sim + fixed tick is what lets a second shard
//! replay the same inputs for anti-cheat (docs/architecture/03-netcode-and-sharding.md).

use std::time::Duration;

use omm_ecs_core::TICK_DT;
use omm_protocol::Intent;
use omm_sim::{simulate, EntityState};

/// Advance one authoritative entity by a batch of intents for this tick.
/// Extracted from the loop so the tick step is unit-tested without a runtime.
fn tick(state: EntityState, inputs: &[Intent]) -> EntityState {
    simulate(state, inputs)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    let period = Duration::from_secs_f32(TICK_DT);
    let mut ticker = tokio::time::interval(period);
    let mut state = EntityState::default();
    tracing::info!("shard tick loop started at {:.1} Hz", 1.0 / TICK_DT);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // No transport yet: drain zero intents and advance the world.
                state = tick(state, &[]);
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("shutting down shard");
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_protocol::Vec3;

    #[test]
    fn tick_applies_movement_intent() {
        let start = EntityState::default();
        let moved = tick(
            start,
            &[Intent::Move {
                dir: Vec3 {
                    x: 3.0,
                    y: 0.0,
                    z: 0.0,
                },
            }],
        );
        // One tick at 30 Hz over velocity 3 => +0.1 on x.
        assert!((moved.pos.0.x - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn empty_tick_is_a_noop_on_position() {
        let start = EntityState::default();
        assert_eq!(tick(start, &[]).pos, start.pos);
    }
}
