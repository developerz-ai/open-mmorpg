//! Economy definitions — auction houses and trading rules.

use serde::{Deserialize, Serialize};

/// Economy data definitions.
// No `Eq`: holds `AuctionHouseDef`, whose fee fields are `f32`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EconomyData {
    /// Auction houses.
    #[serde(default)]
    pub auction_houses: Vec<AuctionHouseDef>,
    /// Trading rules.
    #[serde(default)]
    pub trading_rules: Vec<TradingRuleDef>,
    /// Starting gold for new characters (in copper).
    #[serde(default)]
    pub starting_gold_copper: u32,
}

/// Auction house definition.
// No `Eq`: fee/min-bid/deposit percentages are `f32`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuctionHouseDef {
    /// Stable machine id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Hosting zone id (where the AH physically exists).
    pub zone_id: String,
    /// AH position in zone.
    #[serde(default)]
    pub position: Option<[f32; 3]>,
    /// Fee percentage (0.05 = 5%).
    #[serde(default)]
    pub fee_percentage: f32,
    /// Minimum listing increment (percentage of current bid).
    #[serde(default)]
    pub min_bid_increment: f32,
    /// Maximum active listings per account.
    #[serde(default)]
    pub max_listings_per_account: u16,
    /// Listing duration in hours.
    #[serde(default)]
    pub listing_duration_hours: u32,
    /// Deposit cost percentage (0.05 = 5% of item value).
    #[serde(default)]
    pub deposit_percentage: f32,
}

/// Trading rules for item types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradingRuleDef {
    /// Item pattern this applies to (e.g., "soulbound", "quest_*").
    pub item_pattern: String,
    /// Whether this item can be traded.
    pub tradable: bool,
    /// Whether this item can be auctioned.
    pub auctionable: bool,
    /// Whether this item can be mailed.
    pub mailing_allowed: bool,
}
