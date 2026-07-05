//! Read projections the web consumes: the **armory** (character summaries) and
//! the **auction house** browse/search view.
//!
//! Crucial boundary: these are **reads**. The authoritative auction *buy* moves
//! value through a single `persistence` transaction (economy plumbing) — never
//! here. This board caches listings for browse/search only; it never holds an
//! authoritative quantity (docs/specs/game-server/economy).

use omm_errors::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A character's public summary, as shown in the web armory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSummary {
    /// Character id (raw).
    pub id: u64,
    pub name: String,
    pub level: u16,
    /// Class machine id (from content).
    pub class: String,
}

/// The armory projection: character summaries keyed by id.
#[derive(Debug, Default)]
pub struct Armory {
    chars: BTreeMap<u64, CharacterSummary>,
}

impl Armory {
    /// An empty armory.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or update a character summary.
    pub fn upsert(&mut self, summary: CharacterSummary) {
        self.chars.insert(summary.id, summary);
    }

    /// Fetch one character.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&CharacterSummary> {
        self.chars.get(&id)
    }

    /// Case-insensitive name-prefix search, ordered by id.
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<CharacterSummary> {
        let q = query.to_lowercase();
        self.chars
            .values()
            .filter(|c| c.name.to_lowercase().starts_with(&q))
            .cloned()
            .collect()
    }
}

/// A listing id (worldsvc-issued for the browse projection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ListingId(pub u64);

/// One auction listing, as browsed. Authoritative escrow/quantity lives in
/// `persistence`; this is the searchable view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listing {
    pub id: ListingId,
    /// Item definition machine id (from content).
    pub item_def: String,
    /// Seller character id (raw).
    pub seller: u64,
    /// Unit price in the base currency.
    pub price: u64,
    pub quantity: u32,
}

/// The auction browse projection.
#[derive(Debug, Default)]
pub struct AuctionBoard {
    listings: BTreeMap<ListingId, Listing>,
    next_id: u64,
}

impl AuctionBoard {
    /// An empty board.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a listing in the browse view. Returns its id. (The value escrow is
    /// a `persistence` transaction done before this projection is updated.)
    ///
    /// # Errors
    /// [`CoreError::BadRequest`] on an empty item id or zero quantity/price.
    pub fn list(
        &mut self,
        item_def: &str,
        seller: u64,
        price: u64,
        quantity: u32,
    ) -> CoreResult<ListingId> {
        if item_def.trim().is_empty() || quantity == 0 || price == 0 {
            return Err(CoreError::BadRequest("invalid listing".into()));
        }
        self.next_id += 1;
        let id = ListingId(self.next_id);
        self.listings.insert(
            id,
            Listing {
                id,
                item_def: item_def.to_owned(),
                seller,
                price,
                quantity,
            },
        );
        Ok(id)
    }

    /// Remove a listing from the view (after an authoritative buy/cancel txn).
    pub fn remove(&mut self, id: ListingId) -> Option<Listing> {
        self.listings.remove(&id)
    }

    /// Browse listings, optionally filtered by item-def substring, ordered by id.
    #[must_use]
    pub fn browse(&self, item_filter: Option<&str>) -> Vec<Listing> {
        self.listings
            .values()
            .filter(|l| item_filter.is_none_or(|f| l.item_def.contains(f)))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(id: u64, name: &str) -> CharacterSummary {
        CharacterSummary {
            id,
            name: name.into(),
            level: 10,
            class: "warden".into(),
        }
    }

    #[test]
    fn armory_upsert_get_search() {
        let mut a = Armory::new();
        a.upsert(summary(1, "Ashe"));
        a.upsert(summary(2, "Ashwin"));
        a.upsert(summary(3, "Bran"));
        assert_eq!(a.get(2).unwrap().name, "Ashwin");
        let hits = a.search("ash");
        assert_eq!(hits.iter().map(|c| c.id).collect::<Vec<_>>(), [1, 2]);
    }

    #[test]
    fn auction_list_browse_and_filter() {
        let mut b = AuctionBoard::new();
        b.list("iron-sword", 1, 100, 1).unwrap();
        b.list("iron-shield", 1, 50, 2).unwrap();
        b.list("oak-staff", 2, 200, 1).unwrap();
        assert_eq!(b.browse(None).len(), 3);
        assert_eq!(b.browse(Some("iron")).len(), 2);
    }

    #[test]
    fn auction_rejects_invalid_and_removes() {
        let mut b = AuctionBoard::new();
        assert!(b.list("", 1, 100, 1).is_err());
        assert!(b.list("x", 1, 0, 1).is_err());
        let id = b.list("gem", 1, 10, 1).unwrap();
        assert!(b.remove(id).is_some());
        assert!(b.browse(None).is_empty());
    }
}
