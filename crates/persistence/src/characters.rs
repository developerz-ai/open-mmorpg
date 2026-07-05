//! `characters` repository — position/zone, keyed by newtypes, FK to accounts.
//!
//! Position is [`omm_protocol::Vec3`] (an `f32` triple), the same coordinate
//! type the deterministic simulation and the wire use — one representation, no
//! lossy conversion at the storage edge.

use crate::error::map_sqlx;
use omm_errors::{CoreError, CoreResult};
use omm_protocol::{AccountId, CharacterId, Vec3, ZoneId};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

/// A durable character.
#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    /// Primary key, minted by the database.
    pub id: CharacterId,
    /// Owning account.
    pub account_id: AccountId,
    /// Unique display name.
    pub name: String,
    /// The zone the character is currently in.
    pub zone_id: ZoneId,
    /// World-space position.
    pub position: Vec3,
}

/// The fields needed to create a character.
#[derive(Debug, Clone)]
pub struct NewCharacter<'a> {
    /// Owning account.
    pub account_id: AccountId,
    /// Unique display name.
    pub name: &'a str,
    /// Starting zone.
    pub zone_id: ZoneId,
    /// Starting position.
    pub position: Vec3,
}

/// Reads and writes `characters`. Borrows the pool; holds no other state.
#[derive(Debug, Clone)]
pub struct CharacterRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> CharacterRepo<'a> {
    /// Bind a repository to a pool.
    #[must_use]
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a character, returning its database-minted id.
    ///
    /// # Errors
    /// [`CoreError::Conflict`] if `name` is taken; [`CoreError::NotFound`] via
    /// the FK if `account_id` does not exist.
    pub async fn create(&self, new: &NewCharacter<'_>) -> CoreResult<Character> {
        let id: i64 = sqlx::query(
            "INSERT INTO characters (account_id, name, zone_id, pos_x, pos_y, pos_z) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(new.account_id.raw() as i64)
        .bind(new.name)
        .bind(new.zone_id.raw() as i64)
        .bind(new.position.x)
        .bind(new.position.y)
        .bind(new.position.z)
        .fetch_one(self.pool)
        .await
        .map_err(map_sqlx)?
        .try_get("id")
        .map_err(map_sqlx)?;
        Ok(Character {
            id: CharacterId::new(id as u64),
            account_id: new.account_id,
            name: new.name.to_owned(),
            zone_id: new.zone_id,
            position: new.position,
        })
    }

    /// Fetch a character by id.
    ///
    /// # Errors
    /// [`CoreError::NotFound`] if absent.
    pub async fn find(&self, id: CharacterId) -> CoreResult<Character> {
        let row = sqlx::query(
            "SELECT id, account_id, name, zone_id, pos_x, pos_y, pos_z \
             FROM characters WHERE id = $1",
        )
        .bind(id.raw() as i64)
        .fetch_one(self.pool)
        .await
        .map_err(map_sqlx)?;
        row_to_character(&row)
    }

    /// List an account's characters, ordered by id.
    ///
    /// # Errors
    /// Propagates a mapped [`CoreError`] on a query failure.
    pub async fn list_for_account(&self, account: AccountId) -> CoreResult<Vec<Character>> {
        let rows = sqlx::query(
            "SELECT id, account_id, name, zone_id, pos_x, pos_y, pos_z \
             FROM characters WHERE account_id = $1 ORDER BY id",
        )
        .bind(account.raw() as i64)
        .fetch_all(self.pool)
        .await
        .map_err(map_sqlx)?;
        rows.iter().map(row_to_character).collect()
    }

    /// Persist a character's zone + position — the cadence/logout save path.
    ///
    /// # Errors
    /// [`CoreError::NotFound`] if no character has `id`.
    pub async fn save_position(&self, id: CharacterId, zone: ZoneId, pos: Vec3) -> CoreResult<()> {
        let affected = sqlx::query(
            "UPDATE characters SET zone_id = $2, pos_x = $3, pos_y = $4, pos_z = $5 WHERE id = $1",
        )
        .bind(id.raw() as i64)
        .bind(zone.raw() as i64)
        .bind(pos.x)
        .bind(pos.y)
        .bind(pos.z)
        .execute(self.pool)
        .await
        .map_err(map_sqlx)?
        .rows_affected();
        if affected == 0 {
            return Err(CoreError::NotFound(format!("character {}", id.raw())));
        }
        Ok(())
    }
}

/// Decode one row into a [`Character`]. Ids round-trip u64<->i64 bit-for-bit.
fn row_to_character(row: &PgRow) -> CoreResult<Character> {
    let id: i64 = row.try_get("id").map_err(map_sqlx)?;
    let account_id: i64 = row.try_get("account_id").map_err(map_sqlx)?;
    let name: String = row.try_get("name").map_err(map_sqlx)?;
    let zone_id: i64 = row.try_get("zone_id").map_err(map_sqlx)?;
    let x: f32 = row.try_get("pos_x").map_err(map_sqlx)?;
    let y: f32 = row.try_get("pos_y").map_err(map_sqlx)?;
    let z: f32 = row.try_get("pos_z").map_err(map_sqlx)?;
    Ok(Character {
        id: CharacterId::new(id as u64),
        account_id: AccountId::new(account_id as u64),
        name,
        zone_id: ZoneId::new(zone_id as u64),
        position: Vec3 { x, y, z },
    })
}
