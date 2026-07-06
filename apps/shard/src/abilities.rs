//! Content ability table — the data→runtime bridge for combat.
//!
//! Content authors abilities as `content-schema` data (`content/abilities/*.json`);
//! the sim resolves the compiled [`omm_ecs_core::AbilityDef`] runtime shape. This
//! module is the seam between the two, run once at shard boot: it lowers every
//! authored ability onto the runtime shape and keys them by the same stable
//! [`AbilityId`] the wire `Intent::UseAbility` names, so a cast resolves against
//! exactly the ability the client asked for.
//!
//! **Fail-loud is the contract.** Structurally invalid data — a blank, duplicate,
//! or colliding id, a non-finite/negative/out-of-range number, an unknown target
//! token — returns an error and aborts boot rather than shipping an ability the
//! shard would mis-simulate. Effect *kinds* the runtime does not model yet (buffs,
//! debuffs, summons, teleports, auras) are valid content with no [`EffectKind`]
//! variant: they are skipped, keeping the ability's cost, cooldown, and any
//! modeled damage/heal. Adding the behavior is a compiled change (a new variant +
//! system), never a data error.
//!
//! **Determinism.** The string→id hash and the seconds→ticks rounding are pure and
//! machine-independent, so every shard builds a bit-identical table from the same
//! datapack — the property replay and anti-cheat re-sim stand on.

use std::collections::BTreeMap;

use omm_content_schema::{AbilityDef as ContentAbility, AbilityEffect, AbilityEffectType};
use omm_ecs_core::{AbilityDef, AbilityId, EffectKind, TargetKind, TargetShape, TICK_DT};

/// Engine-default global cooldown in seconds, armed by every ability. Content
/// carries no per-ability GCD yet; this conventional MMO value is applied
/// uniformly until it does.
const DEFAULT_GCD_SECS: f32 = 1.5;

/// FNV-1a 32-bit basis and prime — a stable, dependency-free string hash. Chosen
/// so client and server derive the *same* [`AbilityId`] from an ability's string
/// id with no shared registry.
const FNV_OFFSET: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;

/// The largest value a lossless `f32`→`u32` conversion can round to.
const MAX_U32_AS_F32: f32 = u32::MAX as f32;

/// Why lowering the content ability table failed. Every variant aborts boot — the
/// shard refuses to run a datapack it would mis-simulate.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum AbilityLoadError {
    /// An ability whose trimmed id is empty — it has no stable identity.
    #[error("ability at index {index} has a blank id")]
    BlankId { index: usize },
    /// Two entries share one string id.
    #[error("duplicate ability id '{id}'")]
    DuplicateId { id: String },
    /// Two distinct ids hash to one [`AbilityId`] — casts would be ambiguous.
    #[error("ability ids '{a}' and '{b}' collide at id {hash}")]
    HashCollision { a: String, b: String, hash: u32 },
    /// A number that cannot be lowered: non-finite, negative, or out of range.
    #[error("ability '{id}': field '{field}' is out of range ({value})")]
    BadNumber {
        id: String,
        field: &'static str,
        value: f32,
    },
    /// A target token outside the known vocabulary.
    #[error("ability '{id}': unknown target token '{token}'")]
    UnknownTarget { id: String, token: String },
}

/// The canonical string→[`AbilityId`] map: FNV-1a 32-bit over the id's bytes.
///
/// Pure and machine-independent, so the client can derive the id it names in an
/// intent from the same string the datapack ships — no shared numeric registry.
#[must_use]
pub fn ability_id_hash(id: &str) -> u32 {
    let mut hash = FNV_OFFSET;
    for &byte in id.as_bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Lower validated content abilities onto the runtime table the sim resolves
/// casts against, keyed by the stable [`AbilityId`] the wire names.
///
/// # Errors
/// [`AbilityLoadError`] on any structurally invalid entry (blank/duplicate/
/// colliding id, non-finite/negative/out-of-range number, unknown target token).
pub fn build_ability_table(
    defs: &[ContentAbility],
) -> Result<BTreeMap<AbilityId, AbilityDef>, AbilityLoadError> {
    let mut table = BTreeMap::new();
    let mut by_hash: BTreeMap<u32, String> = BTreeMap::new();
    for (index, def) in defs.iter().enumerate() {
        let id = def.id.trim();
        if id.is_empty() {
            return Err(AbilityLoadError::BlankId { index });
        }
        let hash = ability_id_hash(id);
        if let Some(existing) = by_hash.get(&hash) {
            return Err(if existing == id {
                AbilityLoadError::DuplicateId { id: id.to_owned() }
            } else {
                AbilityLoadError::HashCollision {
                    a: existing.clone(),
                    b: id.to_owned(),
                    hash,
                }
            });
        }
        by_hash.insert(hash, id.to_owned());
        table.insert(AbilityId(hash), lower(AbilityId(hash), id, def)?);
    }
    Ok(table)
}

/// Lower one content ability onto the runtime shape.
fn lower(
    ability_id: AbilityId,
    id: &str,
    def: &ContentAbility,
) -> Result<AbilityDef, AbilityLoadError> {
    let mut effects = Vec::with_capacity(def.effects.len());
    for effect in &def.effects {
        if let Some(kind) = lower_effect(id, effect)? {
            effects.push(kind);
        }
    }
    Ok(AbilityDef {
        id: ability_id,
        power_cost: u32::from(def.resource_cost),
        cooldown_ticks: secs_to_ticks(id, "cooldown_sec", def.cooldown_sec)?,
        gcd_ticks: secs_to_ticks(id, "gcd", DEFAULT_GCD_SECS)?,
        range: finite_non_neg(id, "range_yards", def.range_yards)?,
        target_kind: derive_target_kind(id, &def.effects)?,
        // AoE is authored via `aoe_*`/`party`/`group` tokens but carries no
        // radius yet; a fabricated one would change resolution, so AoE degrades
        // to a single primary target until content ships an explicit radius.
        shape: TargetShape::Single,
        effects,
    })
}

/// Map one content effect onto a runtime [`EffectKind`], or `None` when the
/// runtime models no such kind yet (buff/debuff/summon/teleport/aura). Auras are
/// skipped because content carries no period/duration to build an `AuraSpec`.
fn lower_effect(id: &str, effect: &AbilityEffect) -> Result<Option<EffectKind>, AbilityLoadError> {
    let kind = match effect.effect {
        AbilityEffectType::Damage => EffectKind::Damage(amount(id, effect.magnitude)?),
        AbilityEffectType::Heal => EffectKind::Heal(amount(id, effect.magnitude)?),
        AbilityEffectType::ApplyAura
        | AbilityEffectType::Summon
        | AbilityEffectType::Teleport
        | AbilityEffectType::Buff
        | AbilityEffectType::Debuff
        | AbilityEffectType::Dot
        | AbilityEffectType::Hot => return Ok(None),
    };
    Ok(Some(kind))
}

/// A base magnitude lowered to a flat effect amount. Scaling is *not* applied
/// here — it resolves against caster stats at cast time; the static table holds
/// the authored base only.
fn amount(id: &str, magnitude: f32) -> Result<u32, AbilityLoadError> {
    let base = finite_non_neg(id, "magnitude", magnitude)?;
    if base > MAX_U32_AS_F32 {
        return Err(bad(id, "magnitude", magnitude));
    }
    Ok(base.round() as u32)
}

/// Seconds → whole fixed ticks, rounded to nearest. Fails on a non-finite,
/// negative, or out-of-range duration.
fn secs_to_ticks(id: &str, field: &'static str, secs: f32) -> Result<u32, AbilityLoadError> {
    let ticks = (finite_non_neg(id, field, secs)? / TICK_DT).round();
    if ticks > MAX_U32_AS_F32 {
        return Err(bad(id, field, secs));
    }
    Ok(ticks as u32)
}

/// Guard a finite, non-negative `f32`, returning it unchanged.
fn finite_non_neg(id: &str, field: &'static str, value: f32) -> Result<f32, AbilityLoadError> {
    if !value.is_finite() || value < 0.0 {
        return Err(bad(id, field, value));
    }
    Ok(value)
}

/// Build a [`AbilityLoadError::BadNumber`] for `id`/`field`/`value`.
fn bad(id: &str, field: &'static str, value: f32) -> AbilityLoadError {
    AbilityLoadError::BadNumber {
        id: id.to_owned(),
        field,
        value,
    }
}

/// Derive the ability's single runtime target relation from its effect target
/// tokens, by precedence enemy > ally > self: an ability with any enemy effect is
/// cast at an enemy (self/ally effects ride along). Empty/absent tokens default
/// to self. AoE and party/group tokens keep their relation; the radius is not
/// modeled yet, so the shape stays [`TargetShape::Single`].
fn derive_target_kind(id: &str, effects: &[AbilityEffect]) -> Result<TargetKind, AbilityLoadError> {
    // 0 = self, 1 = ally, 2 = enemy — the precedence order.
    let mut rank = 0u8;
    for effect in effects {
        let token = effect.target.trim().to_ascii_lowercase();
        if token.is_empty() {
            continue;
        }
        let relation = match token.as_str() {
            "self" => 0,
            "ally" | "aoe_ally" | "party" | "group" => 1,
            "enemy" | "aoe_enemy" => 2,
            _ => {
                return Err(AbilityLoadError::UnknownTarget {
                    id: id.to_owned(),
                    token,
                })
            }
        };
        rank = rank.max(relation);
    }
    Ok(match rank {
        2 => TargetKind::Enemy,
        1 => TargetKind::Ally,
        _ => TargetKind::SelfOnly,
    })
}
