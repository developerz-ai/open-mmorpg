//! Shard — a headless authoritative zone server.
//!
//! It owns one slice of the world: it advances the deterministic [`World`] on a
//! fixed tick, applies validated client intents, and (as later slices land)
//! streams snapshots over the netcode transport. Deterministic sim + fixed tick
//! is what lets a second shard replay the same inputs for anti-cheat
//! (docs/architecture/03-netcode-and-sharding.md).

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use omm_content_schema::load_manifest_dir;
use omm_shard::abilities::build_ability_table;
use omm_shard::tick::FixedTimestep;
use omm_sim::{InputBatch, World};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    // The authoritative entity set this shard owns. Actors are admitted as
    // sessions bind (a later slice); it starts empty and advances regardless.
    let mut world = World::new();
    // The content ability table `World::step` resolves casts against, lowered
    // from the datapack at boot. A present-but-invalid datapack is fail-loud
    // (aborts boot); a *missing* one degrades to an empty table so dev and CI
    // run without one — an empty table simply drops every cast cleanly.
    let content_dir = std::env::var("OMM_CONTENT_DIR").unwrap_or_else(|_| "content".to_string());
    let content_dir = Path::new(&content_dir);
    let abilities = if content_dir.join("manifest.json").is_file() {
        let manifest = load_manifest_dir(content_dir)?;
        let table = build_ability_table(&manifest.abilities)?;
        tracing::info!(
            count = table.len(),
            dir = %content_dir.display(),
            "loaded content ability table"
        );
        table
    } else {
        tracing::warn!(
            dir = %content_dir.display(),
            "no datapack found — booting with an empty ability table"
        );
        BTreeMap::new()
    };
    // No transport is wired yet, so every tick drains zero intents. Hoisted out
    // of the loop to keep the tick path allocation-free.
    let inputs: InputBatch = Vec::new();

    let mut timestep = FixedTimestep::new();
    let mut ticker = tokio::time::interval(timestep.period());
    // The accumulator owns catch-up; the pacer must not also burst after a lag
    // spike, or the two would double-count and overshoot.
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut last = Instant::now();
    tracing::info!(
        hz = 1.0 / timestep.period().as_secs_f32(),
        "shard tick loop started"
    );

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let now = Instant::now();
                let elapsed = now.duration_since(last);
                last = now;

                let steps = timestep.advance(elapsed);
                if steps.overran() {
                    tracing::warn!(
                        ran = steps.count,
                        dropped_ms = steps.dropped.as_millis(),
                        "tick overrun: sim fell behind, dropping backlog to stay real-time"
                    );
                }
                for _ in 0..steps.count {
                    world.step(&inputs, &abilities);
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!(tick = world.now().0, "shutting down shard");
                break;
            }
        }
    }
    Ok(())
}
