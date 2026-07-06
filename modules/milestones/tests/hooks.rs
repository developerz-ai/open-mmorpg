//! End-to-end hook coverage for the `milestones` module, driven through its
//! public surface only — the same way the core would.
//!
//! Each activity milestone (explorer, chatterbox, tinkerer) has a boundary test,
//! plus a full-session test that fires every implemented hook and asserts all six
//! milestones unlock. The lock-free essentials (idempotency, heartbeat) are unit
//! tests inside `src/lib.rs`.

use omm_ecs_core::EntityId;
use omm_module_api::{ChatChannel, ChatCtx, ItemUseCtx, Level, LevelUpCtx, ServerHooks};
use omm_module_api::{PlayerLoginCtx, ZoneEnterCtx};
use omm_module_milestones::{Milestone, Milestones};
use omm_protocol::{AccountId, CharacterId, ItemDefId, ItemId, ZoneId};

fn login_ctx() -> PlayerLoginCtx {
    PlayerLoginCtx::new(AccountId::new(1), CharacterId::new(2), EntityId(3))
}

fn level_ctx(from: u16, to: u16) -> LevelUpCtx {
    LevelUpCtx::new(
        EntityId(3),
        CharacterId::new(2),
        Level::new(from),
        Level::new(to),
    )
}

fn zone_ctx() -> ZoneEnterCtx {
    ZoneEnterCtx::new(EntityId(3), CharacterId::new(2), None, ZoneId::new(9))
}

fn item_ctx() -> ItemUseCtx {
    ItemUseCtx::new(EntityId(3), ItemId::new(4), ItemDefId::new(5), None)
}

#[test]
fn explorer_unlocks_after_enough_zone_enters() {
    let m = Milestones::default();
    for _ in 0..4 {
        m.on_zone_enter(&zone_ctx());
    }
    assert!(!m.is_unlocked(Milestone::Explorer));
    m.on_zone_enter(&zone_ctx());
    assert!(m.is_unlocked(Milestone::Explorer));
    assert_eq!(m.zone_enters(), 5);
}

#[test]
fn chatterbox_unlocks_after_enough_chat_lines() {
    let m = Milestones::default();
    let ctx = ChatCtx::new(EntityId(3), ChatChannel::Say, "gg");
    for _ in 0..49 {
        m.on_chat(&ctx);
    }
    assert!(!m.is_unlocked(Milestone::Chatterbox));
    m.on_chat(&ctx);
    assert!(m.is_unlocked(Milestone::Chatterbox));
    assert_eq!(m.chat_lines(), 50);
}

#[test]
fn tinkerer_unlocks_after_enough_item_uses() {
    let m = Milestones::default();
    for _ in 0..9 {
        m.on_item_use(&item_ctx());
    }
    assert!(!m.is_unlocked(Milestone::Tinkerer));
    m.on_item_use(&item_ctx());
    assert!(m.is_unlocked(Milestone::Tinkerer));
    assert_eq!(m.item_uses(), 10);
}

#[test]
fn a_full_session_unlocks_every_milestone() {
    let m = Milestones::default();
    m.on_player_login(&login_ctx());
    m.on_level_up(&level_ctx(1, 20));
    for _ in 0..5 {
        m.on_zone_enter(&zone_ctx());
    }
    let chat = ChatCtx::new(EntityId(3), ChatChannel::Guild, "hi");
    for _ in 0..50 {
        m.on_chat(&chat);
    }
    for _ in 0..10 {
        m.on_item_use(&item_ctx());
    }
    assert_eq!(m.unlocked_count(), Milestone::ALL.len() as u32);
    for milestone in Milestone::ALL {
        assert!(m.is_unlocked(milestone), "{} unlocked", milestone.label());
    }
}
