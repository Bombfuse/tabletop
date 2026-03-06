-- Migration: add "levels" table.
-- This is a follow-up migration for databases that already applied 0001_cards.sql
-- before the "levels" table existed there.

CREATE TABLE IF NOT EXISTS levels (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL UNIQUE,
    text          TEXT NOT NULL,

    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (length(trim(name)) > 0),
    CHECK (length(trim(text)) > 0)
);

CREATE INDEX IF NOT EXISTS idx_levels_name ON levels(name);

CREATE TRIGGER IF NOT EXISTS trg_levels_updated_at
AFTER UPDATE ON levels
FOR EACH ROW
BEGIN
    UPDATE levels
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;
