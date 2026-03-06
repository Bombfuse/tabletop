-- Migration: create simulator "campaigns" table.
--
-- This database is separate from the core tabletop database and is intended to
-- persist simulator-only state (e.g., created campaigns, progress, settings).
--
-- Notes:
-- - Timestamps are stored as TEXT in RFC3339-like UTC format from SQLite.
-- - `hero_unit_name` references a unit card by name in the core DB, but we do
--   not enforce a foreign key because it lives in a separate database file.

CREATE TABLE IF NOT EXISTS campaigns (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    hero_unit_name     TEXT NOT NULL,
    created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (length(trim(hero_unit_name)) > 0)
);

CREATE INDEX IF NOT EXISTS idx_campaigns_created_at ON campaigns(created_at);
CREATE INDEX IF NOT EXISTS idx_campaigns_hero_unit_name ON campaigns(hero_unit_name);
