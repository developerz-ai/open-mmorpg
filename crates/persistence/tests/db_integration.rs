//! Live-database integration tests. Ignored by default so `bin/check` stays
//! green with no database; the `rust-integration` CI job runs them against a
//! real Postgres:
//!
//! ```text
//! TEST_DATABASE_URL=postgres://omm:omm@localhost:5432/omm_test \
//!   cargo nextest run -p omm-persistence --run-ignored all
//! ```

use omm_persistence::{
    connect, health_check, run_migrations, AccountRepo, CharacterRepo, DbConfig, NewCharacter,
    PgPool,
};
use omm_protocol::{Vec3, ZoneId};
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique-per-process suffix so reruns against a persistent dev DB never collide
/// on a UNIQUE column. No wall-clock/RNG needed.
static SEQ: AtomicU64 = AtomicU64::new(0);
fn unique(prefix: &str) -> String {
    format!(
        "{prefix}_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    )
}

/// Connect + migrate, or `None` if `TEST_DATABASE_URL` is unset (skip cleanly).
// A test-only helper: `allow-expect-in-tests` only reaches `#[test]`-annotated
// functions, so this fixture opts in explicitly.
#[allow(clippy::expect_used)]
async fn setup() -> Option<PgPool> {
    let url = std::env::var("TEST_DATABASE_URL").ok()?;
    let cfg = DbConfig::new(url);
    let pool = connect(&cfg).await.expect("connect to the test database");
    run_migrations(&pool).await.expect("run migrations");
    Some(pool)
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL"]
async fn pool_is_healthy_and_migrations_are_idempotent() {
    let Some(pool) = setup().await else { return };
    health_check(&pool).await.expect("health check");
    // Re-running migrations on an already-migrated DB must be a no-op, not drift.
    run_migrations(&pool)
        .await
        .expect("migrations are idempotent");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL"]
async fn account_and_character_round_trip_and_persist() {
    let Some(pool) = setup().await else { return };

    let accounts = AccountRepo::new(&pool);
    let username = unique("hero");
    let created = accounts
        .create(&username, "argon2-placeholder-hash")
        .await
        .unwrap();
    let found = accounts.find_by_username(&username).await.unwrap();
    assert_eq!(created, found, "account round-trips by username");

    let chars = CharacterRepo::new(&pool);
    let name = unique("Aria");
    let made = chars
        .create(&NewCharacter {
            account_id: created.id,
            name: &name,
            zone_id: ZoneId::new(1),
            position: Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        })
        .await
        .unwrap();
    assert_eq!(
        chars.find(made.id).await.unwrap(),
        made,
        "character round-trips by id"
    );
    assert_eq!(
        chars.list_for_account(created.id).await.unwrap(),
        vec![made.clone()]
    );

    // Position survives a save — the relog/handoff persistence path.
    let moved = Vec3 {
        x: 9.0,
        y: 8.0,
        z: 7.0,
    };
    chars
        .save_position(made.id, ZoneId::new(2), moved)
        .await
        .unwrap();
    let reloaded = chars.find(made.id).await.unwrap();
    assert_eq!(reloaded.zone_id, ZoneId::new(2));
    assert_eq!(reloaded.position, moved);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL"]
async fn duplicate_username_is_a_conflict_not_a_second_row() {
    let Some(pool) = setup().await else { return };
    let accounts = AccountRepo::new(&pool);
    let username = unique("twin");
    accounts.create(&username, "h").await.unwrap();
    let err = accounts.create(&username, "h").await.unwrap_err();
    assert_eq!(err.code(), omm_errors::ClientCode::Conflict);
}
