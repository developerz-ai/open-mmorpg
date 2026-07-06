//! The generated registry links the real modules in and dispatches to them.
//!
//! This is the aggregation-layer proof: `omm_modules::load()` — populated by the
//! `build.rs`-generated `register_all` — contains the `hello-world` crate found
//! under `modules/`, and a hook dispatched through the host reaches that real,
//! linked-in module (asserted by downcasting to its concrete type). No core file
//! names `hello-world`; discovery did.

use omm_ecs_core::EntityId;
use omm_module_api::{PlayerLoginCtx, ServerHooks, TickCtx};
use omm_module_hello_world::HelloWorld;
use omm_protocol::{AccountId, CharacterId, Tick};

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
