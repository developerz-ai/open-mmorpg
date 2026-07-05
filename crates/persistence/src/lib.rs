//! Durable, transactional ownership — the anti-dupe heart of the server.
//!
//! **This is the only crate that writes ownership.** Every ownership mutation is
//! a single transaction against YugabyteDB, never a read-modify-write through a
//! cache or a bus. That is anti-dupe rule #1
//! (docs/architecture/04-data-and-consistency.md).
//!
//! The scaffold ships an in-memory [`MemoryOwnership`] reference implementation
//! so the invariant is testable now; the sqlx/Yugabyte implementation lands
//! behind the same [`OwnershipStore`] trait without changing callers.

use omm_errors::{CoreError, CoreResult};
use omm_protocol::{CharacterId, ItemId};
use std::collections::HashMap;

/// Writes durable ownership. The transactional methods are the *only* sanctioned
/// path to move value between characters — atomic, all-or-nothing.
pub trait OwnershipStore {
    /// Who currently owns `item`, if anyone.
    fn owner_of(&self, item: ItemId) -> Option<CharacterId>;

    /// Atomically move `item` from `from` to `to`.
    ///
    /// Either the item was owned by `from` and is now owned by `to`, or nothing
    /// changed and an error is returned. There is no intermediate state in which
    /// the item exists twice — that is what makes duping impossible.
    ///
    /// # Errors
    /// - [`CoreError::NotFound`] if `item` has no owner.
    /// - [`CoreError::Conflict`] if `from` is not the current owner.
    fn transfer(&mut self, item: ItemId, from: CharacterId, to: CharacterId) -> CoreResult<()>;
}

/// In-memory reference implementation. A `HashMap` update is trivially atomic,
/// which mirrors the single-statement transaction the real store will use.
#[derive(Debug, Default)]
pub struct MemoryOwnership {
    owners: HashMap<u64, u64>,
}

impl MemoryOwnership {
    /// An empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed initial ownership (e.g. from a durable load). Not a gameplay path.
    pub fn seed(&mut self, item: ItemId, owner: CharacterId) {
        self.owners.insert(item.raw(), owner.raw());
    }
}

impl OwnershipStore for MemoryOwnership {
    fn owner_of(&self, item: ItemId) -> Option<CharacterId> {
        self.owners.get(&item.raw()).copied().map(CharacterId::new)
    }

    fn transfer(&mut self, item: ItemId, from: CharacterId, to: CharacterId) -> CoreResult<()> {
        match self.owners.get(&item.raw()).copied() {
            None => Err(CoreError::NotFound(format!("item {}", item.raw()))),
            Some(current) if current != from.raw() => {
                Err(CoreError::Conflict("not the current owner".into()))
            }
            Some(_) => {
                self.owners.insert(item.raw(), to.raw());
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_moves_ownership_exactly_once() {
        let mut store = MemoryOwnership::new();
        let (item, alice, bob) = (ItemId::new(1), CharacterId::new(10), CharacterId::new(20));
        store.seed(item, alice);

        store.transfer(item, alice, bob).unwrap();
        assert_eq!(store.owner_of(item), Some(bob));
    }

    #[test]
    fn cannot_transfer_what_you_do_not_own() {
        let mut store = MemoryOwnership::new();
        let (item, alice, bob, mallory) = (
            ItemId::new(1),
            CharacterId::new(10),
            CharacterId::new(20),
            CharacterId::new(99),
        );
        store.seed(item, alice);

        // Mallory tries to move Alice's item — rejected, ownership unchanged.
        let err = store.transfer(item, mallory, bob).unwrap_err();
        assert_eq!(err.code(), omm_errors::ClientCode::Conflict);
        assert_eq!(store.owner_of(item), Some(alice));
    }

    #[test]
    fn transfer_of_unknown_item_is_not_found() {
        let mut store = MemoryOwnership::new();
        let err = store
            .transfer(ItemId::new(404), CharacterId::new(1), CharacterId::new(2))
            .unwrap_err();
        assert_eq!(err.code(), omm_errors::ClientCode::NotFound);
    }
}
