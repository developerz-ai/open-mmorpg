-- 0002_characters — playable characters with position/zone, owned by an account.
--
-- Position is an f32 triple (REAL), matching the deterministic simulation's
-- coordinate type (`omm_protocol::Vec3`). Transient sim state (velocity,
-- in-flight cast) is NEVER durable — it is rebuilt from this on shard handoff.

CREATE TABLE IF NOT EXISTS characters (
    id         BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id BIGINT      NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    name       TEXT        NOT NULL UNIQUE,
    zone_id    BIGINT      NOT NULL DEFAULT 0,
    pos_x      REAL        NOT NULL DEFAULT 0,
    pos_y      REAL        NOT NULL DEFAULT 0,
    pos_z      REAL        NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS characters_account_id_idx ON characters (account_id);
