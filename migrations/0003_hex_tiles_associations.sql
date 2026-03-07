-- Migration: extend hex_tiles with association/data columns.
--
-- Adds optional associations so a hex tile can reference a Unit, Item, or Level,
-- and can store an optional freeform `type` string.
--
-- NOTE:
-- - Existing databases created before these columns existed will not have them.
-- - This migration is additive and uses SQLite-compatible ALTER TABLE statements.
-- - We also add indexes to keep lookups/snappy editor loads fast.

PRAGMA foreign_keys = ON;

----------------------------------------------------------------------
-- Add columns (idempotent-ish)
--
-- SQLite does not support "ADD COLUMN IF NOT EXISTS" until very recent versions,
-- so we rely on the migration runner's "apply once" behavior.
----------------------------------------------------------------------

ALTER TABLE hex_tiles ADD COLUMN unit_id  INTEGER NULL;
ALTER TABLE hex_tiles ADD COLUMN item_id  INTEGER NULL;
ALTER TABLE hex_tiles ADD COLUMN level_id INTEGER NULL;
ALTER TABLE hex_tiles ADD COLUMN type     TEXT NULL;

----------------------------------------------------------------------
-- Indexes to support common queries
----------------------------------------------------------------------

CREATE INDEX IF NOT EXISTS idx_hex_tiles_unit_id
ON hex_tiles(unit_id);

CREATE INDEX IF NOT EXISTS idx_hex_tiles_item_id
ON hex_tiles(item_id);

CREATE INDEX IF NOT EXISTS idx_hex_tiles_level_id
ON hex_tiles(level_id);

CREATE INDEX IF NOT EXISTS idx_hex_tiles_type
ON hex_tiles(type);

----------------------------------------------------------------------
-- Optional: validate references on write (soft FK via triggers)
--
-- We cannot add FOREIGN KEY constraints to an existing table in SQLite
-- without rebuilding the table, so we enforce integrity with triggers.
----------------------------------------------------------------------

CREATE TRIGGER IF NOT EXISTS trg_hex_tiles_validate_links_insert
BEFORE INSERT ON hex_tiles
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN NEW.unit_id IS NOT NULL
                 AND (SELECT id FROM units WHERE id = NEW.unit_id) IS NULL
            THEN RAISE(ABORT, 'hex_tiles.unit_id references missing units.id')
        END;

    SELECT
        CASE
            WHEN NEW.item_id IS NOT NULL
                 AND (SELECT id FROM items WHERE id = NEW.item_id) IS NULL
            THEN RAISE(ABORT, 'hex_tiles.item_id references missing items.id')
        END;

    SELECT
        CASE
            WHEN NEW.level_id IS NOT NULL
                 AND (SELECT id FROM levels WHERE id = NEW.level_id) IS NULL
            THEN RAISE(ABORT, 'hex_tiles.level_id references missing levels.id')
        END;
END;

CREATE TRIGGER IF NOT EXISTS trg_hex_tiles_validate_links_update
BEFORE UPDATE OF unit_id, item_id, level_id ON hex_tiles
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN NEW.unit_id IS NOT NULL
                 AND (SELECT id FROM units WHERE id = NEW.unit_id) IS NULL
            THEN RAISE(ABORT, 'hex_tiles.unit_id references missing units.id')
        END;

    SELECT
        CASE
            WHEN NEW.item_id IS NOT NULL
                 AND (SELECT id FROM items WHERE id = NEW.item_id) IS NULL
            THEN RAISE(ABORT, 'hex_tiles.item_id references missing items.id')
        END;

    SELECT
        CASE
            WHEN NEW.level_id IS NOT NULL
                 AND (SELECT id FROM levels WHERE id = NEW.level_id) IS NULL
            THEN RAISE(ABORT, 'hex_tiles.level_id references missing levels.id')
        END;
END;

----------------------------------------------------------------------
-- No ON DELETE behavior:
-- If a referenced card is deleted, tiles keep their ids unless you clear them.
-- If you want "ON DELETE SET NULL"-like behavior, we can add delete triggers
-- on units/items/levels to null out matching hex_tiles.*_id.
----------------------------------------------------------------------
