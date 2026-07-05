//! `accounts` repository — the auth boundary. Newtype-keyed; no raw `u64` in
//! the public surface. Credentials are written but never read back into the
//! model (the hash stays in the database).

use crate::error::map_sqlx;
use omm_errors::CoreResult;
use omm_protocol::AccountId;
use sqlx::{PgPool, Row};

/// A durable account, minus its credentials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// Primary key, minted by the database.
    pub id: AccountId,
    /// Unique login name.
    pub username: String,
}

/// Reads and writes `accounts`. Borrows the pool; holds no other state.
#[derive(Debug, Clone)]
pub struct AccountRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> AccountRepo<'a> {
    /// Bind a repository to a pool.
    #[must_use]
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create an account, returning its database-minted id.
    ///
    /// `password_hash` is the already-hashed credential (argon2 at the gateway);
    /// this crate never sees or stores a plaintext password.
    ///
    /// # Errors
    /// [`CoreError::Conflict`](omm_errors::CoreError::Conflict) if `username`
    /// already exists.
    pub async fn create(&self, username: &str, password_hash: &str) -> CoreResult<Account> {
        let id: i64 = sqlx::query(
            "INSERT INTO accounts (username, password_hash) VALUES ($1, $2) RETURNING id",
        )
        .bind(username)
        .bind(password_hash)
        .fetch_one(self.pool)
        .await
        .map_err(map_sqlx)?
        .try_get("id")
        .map_err(map_sqlx)?;
        Ok(Account {
            id: AccountId::new(id as u64),
            username: username.to_owned(),
        })
    }

    /// Look up an account by login name.
    ///
    /// # Errors
    /// [`CoreError::NotFound`](omm_errors::CoreError::NotFound) if absent.
    pub async fn find_by_username(&self, username: &str) -> CoreResult<Account> {
        let row = sqlx::query("SELECT id, username FROM accounts WHERE username = $1")
            .bind(username)
            .fetch_one(self.pool)
            .await
            .map_err(map_sqlx)?;
        let id: i64 = row.try_get("id").map_err(map_sqlx)?;
        let username: String = row.try_get("username").map_err(map_sqlx)?;
        Ok(Account {
            id: AccountId::new(id as u64),
            username,
        })
    }
}
