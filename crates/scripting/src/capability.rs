//! The **capability API** — the entire surface a sandboxed script can reach.
//!
//! A script gets nothing but the methods on [`HostCapabilities`]. There is no
//! filesystem, network, cache, clock, or database handle anywhere in this
//! surface, and the WASM linker refuses any import the script names beyond these
//! functions (see [`crate::wasm`]). This is the boundary; it is deliberately
//! narrow.
//!
//! **Ownership writes never happen inside the sandbox.** [`HostCapabilities::grant_item`]
//! is the seam where the host will later route an item grant through
//! `omm-persistence` in a single YugabyteDB transaction — the script only
//! expresses *intent*, exactly like a client. The trait keeps that asymmetry
//! visible: the sandbox can ask, only the host (on the outside) can commit.

use crate::error::{ScriptError, ScriptResult};
use omm_protocol::ItemId;

/// A live entity handle inside the sim, as seen by a script. Opaque: a script
/// receives one from [`HostCapabilities::spawn_entity`] or a query and can only
/// pass it back to the host — it cannot forge state from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(u64);

impl EntityId {
    /// Wrap a raw handle. Minted by the host, never by a script.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The underlying raw value — used only at the WASM ABI edge.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// A gameplay effect template id (buff/debuff/damage-over-time …). Pure content
/// data; the host resolves it against the loaded datapack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EffectId(u32);

impl EffectId {
    /// Wrap a raw effect template id.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// The underlying raw value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Everything a sandboxed content script is allowed to do.
///
/// Implementations run **on the host side**, outside the sandbox, and are the
/// only code with access to real state. Keep every method total and fast: a
/// script can call them in a hot loop until its fuel runs out.
pub trait HostCapabilities: Send {
    /// Spawn a `kind` of entity at world coordinates `(x, y)`; returns its new
    /// handle. `kind` is a content-defined discriminant resolved by the host.
    fn spawn_entity(&mut self, kind: i64, x: i64, y: i64) -> EntityId;

    /// Count entities within `radius` of `(x, y)`. Read-only by contract.
    fn query_nearby(&self, x: i64, y: i64, radius: i64) -> u32;

    /// Apply `effect` with `magnitude` to `target`.
    fn apply_effect(&mut self, target: EntityId, effect: EffectId, magnitude: i64);

    /// Express intent to grant `qty` of `item` to `target`.
    ///
    /// This is *not* an ownership write. The host implementation commits it
    /// through `omm-persistence` in a single transaction after the script
    /// returns; here it only records the request and may reject it.
    ///
    /// # Errors
    /// [`ScriptError::CapabilityDenied`] if the grant is invalid (e.g. `qty` is
    /// zero) — the script sees a non-zero status and can react, but cannot force
    /// the write.
    fn grant_item(&mut self, target: EntityId, item: ItemId, qty: u32) -> ScriptResult<()>;
}

/// A record of one capability call, for tests and audit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostCall {
    /// A [`HostCapabilities::spawn_entity`] call and the handle it returned.
    Spawn {
        /// Content-defined entity discriminant.
        kind: i64,
        /// World X.
        x: i64,
        /// World Y.
        y: i64,
        /// Handle handed back to the script.
        entity: EntityId,
    },
    /// An [`HostCapabilities::apply_effect`] call.
    Effect {
        /// Effect target.
        target: EntityId,
        /// Effect template.
        effect: EffectId,
        /// Effect magnitude.
        magnitude: i64,
    },
    /// A [`HostCapabilities::grant_item`] intent.
    Grant {
        /// Grant recipient.
        target: EntityId,
        /// Item template.
        item: ItemId,
        /// Quantity requested.
        qty: u32,
    },
}

/// A deterministic, side-effect-free reference host used in tests and examples.
///
/// It records every capability call, mints entity handles from a counter, and
/// answers [`HostCapabilities::query_nearby`] with a fixed `nearby_count`. It
/// touches no external system — it is exactly what "the sandbox can reach
/// nothing but this trait" looks like in practice.
#[derive(Debug, Default)]
pub struct RecordingHost {
    calls: Vec<HostCall>,
    next_entity: u64,
    nearby_count: u32,
}

impl RecordingHost {
    /// A fresh host that reports `nearby_count` for every proximity query.
    #[must_use]
    pub fn new(nearby_count: u32) -> Self {
        Self {
            calls: Vec::new(),
            next_entity: 1,
            nearby_count,
        }
    }

    /// The ordered log of capability calls the script made.
    #[must_use]
    pub fn calls(&self) -> &[HostCall] {
        &self.calls
    }
}

impl HostCapabilities for RecordingHost {
    fn spawn_entity(&mut self, kind: i64, x: i64, y: i64) -> EntityId {
        let entity = EntityId::new(self.next_entity);
        self.next_entity += 1;
        self.calls.push(HostCall::Spawn { kind, x, y, entity });
        entity
    }

    fn query_nearby(&self, _x: i64, _y: i64, _radius: i64) -> u32 {
        self.nearby_count
    }

    fn apply_effect(&mut self, target: EntityId, effect: EffectId, magnitude: i64) {
        self.calls.push(HostCall::Effect {
            target,
            effect,
            magnitude,
        });
    }

    fn grant_item(&mut self, target: EntityId, item: ItemId, qty: u32) -> ScriptResult<()> {
        if qty == 0 {
            return Err(ScriptError::CapabilityDenied(
                "grant quantity is zero".into(),
            ));
        }
        self.calls.push(HostCall::Grant { target, item, qty });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_host_mints_and_logs() {
        let mut host = RecordingHost::new(3);
        let e = host.spawn_entity(7, 1, 2);
        assert_eq!(e, EntityId::new(1));
        assert_eq!(host.query_nearby(0, 0, 10), 3);
        host.apply_effect(e, EffectId::new(5), 12);
        host.grant_item(e, ItemId::new(100), 2).unwrap();
        assert_eq!(host.calls().len(), 3);
    }

    #[test]
    fn zero_grant_is_denied() {
        let mut host = RecordingHost::new(0);
        let e = host.spawn_entity(0, 0, 0);
        let err = host.grant_item(e, ItemId::new(1), 0).unwrap_err();
        assert_eq!(err.code(), omm_errors::ClientCode::Forbidden);
    }
}
