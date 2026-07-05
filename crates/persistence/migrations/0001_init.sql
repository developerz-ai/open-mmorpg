-- 0001_init — durable baseline: accounts (the auth boundary).
--
-- Forward-only. Migrations are NEVER edited after ship — supersede with a new
-- numbered file instead (a checksum change on an applied migration is a hard
-- error, which is how we detect drift in CI).
--
-- Keys are BIGINT: Postgres/Yugabyte have no unsigned integer, so the u64
-- newtypes (`AccountId`, ...) are stored bit-for-bit as i64 at the edge. Ids are
-- minted by the database (GENERATED ALWAYS AS IDENTITY) — one source of truth.

CREATE TABLE IF NOT EXISTS accounts (
    id            BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username      TEXT        NOT NULL UNIQUE,
    password_hash TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
