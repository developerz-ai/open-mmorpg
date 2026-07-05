//! Newtype identifiers. Never pass a raw `u64` where a typed id belongs — the
//! type system is the first anti-cheat layer (you cannot hand an `ItemId` to a
//! function expecting an `AccountId`).

use serde::{Deserialize, Serialize};

/// Declares a `u64` newtype id with the common derives and accessors.
macro_rules! id_newtype {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(pub u64);

        impl $name {
            /// Wrap a raw value. Prefer minting ids at their source of truth.
            #[must_use]
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            /// The underlying raw value — use only at storage/wire edges.
            #[must_use]
            pub const fn raw(self) -> u64 {
                self.0
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

id_newtype!(
    /// A player account. Owns characters; the auth boundary.
    AccountId
);
id_newtype!(
    /// A single playable character belonging to an account.
    CharacterId
);
id_newtype!(
    /// A concrete item instance (not an item template).
    ItemId
);
id_newtype!(
    /// A zone shard process in the topology.
    ShardId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_not_interchangeable_but_roundtrip() {
        let a = AccountId::new(42);
        assert_eq!(a.raw(), 42);
        assert_eq!(a, AccountId(42));
    }

    #[test]
    fn display_names_the_type() {
        assert_eq!(ItemId::new(7).to_string(), "ItemId(7)");
    }
}
