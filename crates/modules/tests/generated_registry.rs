//! The generated registry links the real modules in and dispatches to them.
//!
//! This is the aggregation-layer proof: `omm_modules::load()` — populated by the
//! `build.rs`-generated `register_all` — contains the `hello-world` crate found
//! under `modules/`, and a hook dispatched through the host reaches that real,
//! linked-in module (asserted by downcasting to its concrete type). No core file
//! names `hello-world`; discovery did.

use omm_ecs_core::EntityId;
use omm_module_api::{
    ChatChannel, ChatCtx, ItemUseCtx, Level, LevelUpCtx, PlayerLoginCtx, ServerHooks, TickCtx,
    ZoneEnterCtx,
};
use omm_module_hello_world::HelloWorld;
use omm_module_milestones::{Milestone, Milestones};
use omm_protocol::{AccountId, CharacterId, ItemDefId, ItemId, Tick, ZoneId};

#[test]
fn load_discovers_the_hello_world_module() {
    let host = omm_modules::load();
    assert!(
        !host.is_empty(),
        "at least the hello-world module is linked in"
    );
    let names: Vec<_> = host.manifests().map(|m| m.name).collect();
    assert!(
        names.contains(&"hello-world"),
        "generated registry links hello-world; found {names:?}",
    );
}

#[test]
fn login_dispatched_through_the_host_reaches_the_real_module() {
    let host = omm_modules::load();
    let ctx = PlayerLoginCtx::new(AccountId::new(1), CharacterId::new(2), EntityId(3));

    host.on_player_login(&ctx);

    let module = host
        .get::<HelloWorld>()
        .expect("hello-world is linked in and downcastable");
    assert_eq!(
        module.logins(),
        1,
        "the login hook fired on the real module"
    );
}

#[test]
fn tick_dispatched_through_the_host_reaches_the_real_module() {
    let host = omm_modules::load();

    host.on_tick(&TickCtx::new(Tick(1), 1.0 / 30.0));

    let module = host.get::<HelloWorld>().expect("hello-world is linked in");
    assert_eq!(module.ticks(), 1, "the tick hook fired on the real module");
}

#[test]
fn load_also_discovers_the_milestones_module() {
    let host = omm_modules::load();
    let names: Vec<_> = host.manifests().map(|m| m.name).collect();
    assert!(
        names.contains(&"milestones"),
        "generated registry links milestones; found {names:?}",
    );
}

#[test]
fn several_events_unlock_milestones_on_the_real_module() {
    let host = omm_modules::load();

    // A different hook per event — the whole point of the second module is that
    // several of the added hooks fan out through the host to one linked-in crate.
    host.on_player_login(&PlayerLoginCtx::new(
        AccountId::new(1),
        CharacterId::new(2),
        EntityId(3),
    ));
    host.on_level_up(&LevelUpCtx::new(
        EntityId(3),
        CharacterId::new(2),
        Level::new(1),
        Level::new(10),
    ));

    let module = host
        .get::<Milestones>()
        .expect("milestones is linked in and downcastable");
    assert!(module.is_unlocked(Milestone::FirstLogin));
    assert!(module.is_unlocked(Milestone::Reached10));
    assert_eq!(module.highest_level(), 10);
}

#[test]
fn level_up_to_10_through_host_records_highest_level_and_unlocks_reached10() {
    let host = omm_modules::load();

    host.on_level_up(&LevelUpCtx::new(
        EntityId(4),
        CharacterId::new(5),
        Level::new(1),
        Level::new(10),
    ));

    let module = host.get::<Milestones>().expect("milestones is linked in");
    assert_eq!(
        module.highest_level(),
        10,
        "highest_level tracks the to level"
    );
    assert!(
        module.is_unlocked(Milestone::Reached10),
        "Reached10 unlocked at level 10"
    );
    assert!(
        !module.is_unlocked(Milestone::Reached20),
        "Reached20 not yet unlocked"
    );
}

#[test]
fn zone_enter_chat_item_use_counters_advance_through_host() {
    let host = omm_modules::load();

    host.on_zone_enter(&ZoneEnterCtx::new(
        EntityId(1),
        CharacterId::new(1),
        None,
        ZoneId::new(42),
    ));
    host.on_chat(&ChatCtx::new(EntityId(1), ChatChannel::Say, "hello world"));
    host.on_item_use(&ItemUseCtx::new(
        EntityId(1),
        ItemId::new(1),
        ItemDefId::new(10),
        None,
    ));

    let module = host.get::<Milestones>().expect("milestones is linked in");
    assert_eq!(module.zone_enters(), 1, "on_zone_enter reached milestones");
    assert_eq!(module.chat_lines(), 1, "on_chat reached milestones");
    assert_eq!(module.item_uses(), 1, "on_item_use reached milestones");
}
