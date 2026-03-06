-- Migration: create core "cards" tables: units and items.
-- Notes:
-- - We store stats as INTEGER.
-- - We keep names UNIQUE to allow simple upsert-by-name semantics in the API layer.

CREATE TABLE IF NOT EXISTS units (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL UNIQUE,

    strength      INTEGER NOT NULL,
    focus         INTEGER NOT NULL,
    intelligence  INTEGER NOT NULL,
    agility       INTEGER NOT NULL,
    knowledge     INTEGER NOT NULL,

    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (length(trim(name)) > 0)
);

CREATE INDEX IF NOT EXISTS idx_units_name ON units(name);

CREATE TRIGGER IF NOT EXISTS trg_units_updated_at
AFTER UPDATE ON units
FOR EACH ROW
BEGIN
    UPDATE units
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS items (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL UNIQUE,

    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (length(trim(name)) > 0)
);

CREATE INDEX IF NOT EXISTS idx_items_name ON items(name);

CREATE TRIGGER IF NOT EXISTS trg_items_updated_at
AFTER UPDATE ON items
FOR EACH ROW
BEGIN
    UPDATE items
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;
