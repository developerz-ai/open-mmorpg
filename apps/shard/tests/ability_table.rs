//! Ability-table loader tests: the data→runtime lowering, its fail-loud gates,
//! and an end-to-end guard over the *real* committed datapack.
//!
//! The loader's whole surface is public (`build_ability_table`,
//! `ability_id_hash`), so these live as integration tests rather than inline unit
//! tests. The final case loads `content/` and proves the whole shipped set —
//! including abilities that use effect kinds and target tokens the runtime does
//! not model yet (buffs, summons, `aoe_*`, `party`) — lowers without a boot
//! failure.

use std::path::PathBuf;

use omm_content_schema::{
    load_manifest_dir, AbilityDef as ContentAbility, AbilityEffect, AbilityEffectType,
};
use omm_ecs_core::{AbilityId, EffectKind, TargetKind, TargetShape};
use omm_shard::abilities::{ability_id_hash, build_ability_table, AbilityLoadError};

/// A single-effect content ability with sensible defaults, for lowering tests.
fn content(id: &str, effect: AbilityEffectType, magnitude: f32, target: &str) -> ContentAbility {
    ContentAbility {
        id: id.to_owned(),
        name: id.to_owned(),
        description: String::new(),
        icon: None,
        max_rank: 1,
        cooldown_sec: 0.0,
        resource_cost: 10,
        cast_time_sec: 0.0,
        range_yards: 5.0,
        effects: vec![AbilityEffect {
            effect,
            magnitude,
            scaling: 1.0,
            target: target.to_owned(),
        }],
    }
}

#[test]
fn fnv_hash_is_stable_and_distinguishes() {
    // Pinned vectors: changing these silently rebinds every client's ability
    // ids, so a refactor that shifts them must fail here.
    assert_eq!(ability_id_hash("primal-strike"), 0xc332_fbdd);
    assert_eq!(ability_id_hash("void-bolt"), 0xfec9_f9b1);
    assert_ne!(ability_id_hash("a"), ability_id_hash("b"));
}

#[test]
fn lowers_damage_ability_faithfully() {
    let table = build_ability_table(&[content("nuke", AbilityEffectType::Damage, 13.0, "enemy")])
        .expect("valid");
    let def = table
        .get(&AbilityId(ability_id_hash("nuke")))
        .expect("present");
    assert_eq!(def.power_cost, 10);
    assert_eq!(def.range, 5.0);
    assert_eq!(def.target_kind, TargetKind::Enemy);
    assert_eq!(def.shape, TargetShape::Single);
    assert_eq!(def.effects, vec![EffectKind::Damage(13)]);
    // 1.5s GCD at 30 Hz = 45 ticks; no per-ability cooldown authored.
    assert_eq!(def.gcd_ticks, 45);
    assert_eq!(def.cooldown_ticks, 0);
}

#[test]
fn cooldown_seconds_round_to_nearest_tick() {
    let mut a = content("cd", AbilityEffectType::Damage, 1.0, "enemy");
    a.cooldown_sec = 2.0; // 2s * 30 = 60 ticks
    let table = build_ability_table(&[a]).expect("valid");
    assert_eq!(table[&AbilityId(ability_id_hash("cd"))].cooldown_ticks, 60);
}

#[test]
fn heal_and_self_target_lower() {
    let table = build_ability_table(&[content("mend", AbilityEffectType::Heal, 20.0, "self")])
        .expect("valid");
    let def = &table[&AbilityId(ability_id_hash("mend"))];
    assert_eq!(def.effects, vec![EffectKind::Heal(20)]);
    assert_eq!(def.target_kind, TargetKind::SelfOnly);
}

#[test]
fn unmodeled_effect_kinds_are_skipped_not_failed() {
    // A strike that also summons: the Summon has no runtime variant, so only the
    // Damage survives — the ability still loads with its cost/cooldown.
    let a = ContentAbility {
        effects: vec![
            AbilityEffect {
                effect: AbilityEffectType::Damage,
                magnitude: 8.0,
                scaling: 0.0,
                target: "enemy".to_owned(),
            },
            AbilityEffect {
                effect: AbilityEffectType::Summon,
                magnitude: 1.0,
                scaling: 0.0,
                target: "self".to_owned(),
            },
        ],
        ..content("summoner", AbilityEffectType::Damage, 8.0, "enemy")
    };
    let table = build_ability_table(&[a]).expect("valid");
    let def = &table[&AbilityId(ability_id_hash("summoner"))];
    assert_eq!(def.effects, vec![EffectKind::Damage(8)]);
    // Enemy wins precedence over the self-targeted summon.
    assert_eq!(def.target_kind, TargetKind::Enemy);
}

#[test]
fn aoe_and_party_tokens_keep_relation_but_stay_single() {
    let aoe = build_ability_table(&[content(
        "blast",
        AbilityEffectType::Damage,
        5.0,
        "aoe_enemy",
    )])
    .expect("valid");
    let def = &aoe[&AbilityId(ability_id_hash("blast"))];
    assert_eq!(def.target_kind, TargetKind::Enemy);
    assert_eq!(def.shape, TargetShape::Single);

    let party = build_ability_table(&[content("aura", AbilityEffectType::Heal, 3.0, "party")])
        .expect("valid");
    assert_eq!(
        party[&AbilityId(ability_id_hash("aura"))].target_kind,
        TargetKind::Ally
    );
}

#[test]
fn blank_id_fails_loud() {
    let err = build_ability_table(&[content("   ", AbilityEffectType::Damage, 1.0, "enemy")])
        .unwrap_err();
    assert_eq!(err, AbilityLoadError::BlankId { index: 0 });
}

#[test]
fn duplicate_id_fails_loud() {
    let dup = [
        content("dup", AbilityEffectType::Damage, 1.0, "enemy"),
        content("dup", AbilityEffectType::Heal, 1.0, "ally"),
    ];
    assert_eq!(
        build_ability_table(&dup).unwrap_err(),
        AbilityLoadError::DuplicateId {
            id: "dup".to_owned()
        }
    );
}

#[test]
fn unknown_target_token_fails_loud() {
    let err = build_ability_table(&[content("warp", AbilityEffectType::Damage, 1.0, "point")])
        .unwrap_err();
    assert_eq!(
        err,
        AbilityLoadError::UnknownTarget {
            id: "warp".to_owned(),
            token: "point".to_owned(),
        }
    );
}

#[test]
fn negative_and_nonfinite_numbers_fail_loud() {
    let mut neg = content("bad", AbilityEffectType::Damage, 1.0, "enemy");
    neg.cooldown_sec = -1.0;
    assert!(matches!(
        build_ability_table(std::slice::from_ref(&neg)).unwrap_err(),
        AbilityLoadError::BadNumber {
            field: "cooldown_sec",
            ..
        }
    ));

    let nan = content("nan", AbilityEffectType::Damage, f32::NAN, "enemy");
    assert!(matches!(
        build_ability_table(&[nan]).unwrap_err(),
        AbilityLoadError::BadNumber {
            field: "magnitude",
            ..
        }
    ));
}

#[test]
fn empty_input_yields_empty_table() {
    assert!(build_ability_table(&[]).expect("valid").is_empty());
}

/// Resolve `content/` relative to this crate (`apps/shard` → workspace root).
fn content_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content")
}

#[test]
fn committed_datapack_lowers_to_runtime_table() {
    let manifest = load_manifest_dir(&content_dir()).expect("content/ must load");
    assert!(
        !manifest.abilities.is_empty(),
        "datapack should ship abilities"
    );

    let table = build_ability_table(&manifest.abilities).expect("datapack must lower fail-free");

    // Every ability lowers (ids are unique — no collisions in the real set).
    assert_eq!(
        table.len(),
        manifest.abilities.len(),
        "every authored ability must appear once in the runtime table"
    );

    // A known melee striker keeps its authored damage and enemy targeting.
    let strike = table
        .get(&AbilityId(ability_id_hash("primal-strike")))
        .expect("primal-strike present");
    assert_eq!(strike.target_kind, TargetKind::Enemy);
    assert_eq!(strike.effects, vec![EffectKind::Damage(13)]);
}
