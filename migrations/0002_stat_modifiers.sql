-- Migration: add stat modifiers + link tables for existing databases.
--
-- Adds:
-- - stat_modifiers
-- - unit_stat_modifiers
-- - item_stat_modifiers
-- - level_stat_modifiers
--
-- Also adds triggers to enforce that a stat modifier can be linked to at most one of:
-- Unit, Item, or Level.

PRAGMA foreign_keys = ON;

----------------------------------------------------------------------
-- Stat modifiers
----------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS stat_modifiers (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,

    stat          TEXT NOT NULL,
    value         INTEGER NOT NULL,
    operator      TEXT NOT NULL,

    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (stat IN ('Strength', 'Focus', 'Intelligence', 'Knowledge', 'Agility')),
    CHECK (operator IN ('Add', 'Subtract'))
);

CREATE INDEX IF NOT EXISTS idx_stat_modifiers_stat ON stat_modifiers(stat);
CREATE INDEX IF NOT EXISTS idx_stat_modifiers_operator ON stat_modifiers(operator);

CREATE TRIGGER IF NOT EXISTS trg_stat_modifiers_updated_at
AFTER UPDATE ON stat_modifiers
FOR EACH ROW
BEGIN
    UPDATE stat_modifiers
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;

----------------------------------------------------------------------
-- Link tables (a stat modifier may be linked to exactly one card type)
----------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS unit_stat_modifiers (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    unit_id           INTEGER NOT NULL,
    stat_modifier_id  INTEGER NOT NULL UNIQUE,

    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (unit_id) REFERENCES units(id) ON DELETE CASCADE,
    FOREIGN KEY (stat_modifier_id) REFERENCES stat_modifiers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_unit_stat_modifiers_unit_id
ON unit_stat_modifiers(unit_id);

CREATE TABLE IF NOT EXISTS item_stat_modifiers (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id           INTEGER NOT NULL,
    stat_modifier_id  INTEGER NOT NULL UNIQUE,

    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
    FOREIGN KEY (stat_modifier_id) REFERENCES stat_modifiers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_item_stat_modifiers_item_id
ON item_stat_modifiers(item_id);

CREATE TABLE IF NOT EXISTS level_stat_modifiers (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    level_id          INTEGER NOT NULL,
    stat_modifier_id  INTEGER NOT NULL UNIQUE,

    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (level_id) REFERENCES levels(id) ON DELETE CASCADE,
    FOREIGN KEY (stat_modifier_id) REFERENCES stat_modifiers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_level_stat_modifiers_level_id
ON level_stat_modifiers(level_id);

----------------------------------------------------------------------
-- Exclusivity triggers: enforce "linked to at most one of unit/item/level"
----------------------------------------------------------------------

CREATE TRIGGER IF NOT EXISTS trg_stat_modifier_link_exclusive_unit
BEFORE INSERT ON unit_stat_modifiers
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN EXISTS (
                SELECT 1 FROM item_stat_modifiers ism WHERE ism.stat_modifier_id = NEW.stat_modifier_id
            )
            OR EXISTS (
                SELECT 1 FROM level_stat_modifiers lsm WHERE lsm.stat_modifier_id = NEW.stat_modifier_id
            )
            THEN RAISE(ABORT, 'stat_modifier already linked to an item or level')
        END;
END;

CREATE TRIGGER IF NOT EXISTS trg_stat_modifier_link_exclusive_item
BEFORE INSERT ON item_stat_modifiers
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN EXISTS (
                SELECT 1 FROM unit_stat_modifiers usm WHERE usm.stat_modifier_id = NEW.stat_modifier_id
            )
            OR EXISTS (
                SELECT 1 FROM level_stat_modifiers lsm WHERE lsm.stat_modifier_id = NEW.stat_modifier_id
            )
            THEN RAISE(ABORT, 'stat_modifier already linked to a unit or level')
        END;
END;

CREATE TRIGGER IF NOT EXISTS trg_stat_modifier_link_exclusive_level
BEFORE INSERT ON level_stat_modifiers
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN EXISTS (
                SELECT 1 FROM unit_stat_modifiers usm WHERE usm.stat_modifier_id = NEW.stat_modifier_id
            )
            OR EXISTS (
                SELECT 1 FROM item_stat_modifiers ism WHERE ism.stat_modifier_id = NEW.stat_modifier_id
            )
            THEN RAISE(ABORT, 'stat_modifier already linked to a unit or item')
        END;
END;

CREATE TRIGGER IF NOT EXISTS trg_stat_modifier_link_exclusive_unit_update
BEFORE UPDATE ON unit_stat_modifiers
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN EXISTS (
                SELECT 1 FROM item_stat_modifiers ism WHERE ism.stat_modifier_id = NEW.stat_modifier_id
            )
            OR EXISTS (
                SELECT 1 FROM level_stat_modifiers lsm WHERE lsm.stat_modifier_id = NEW.stat_modifier_id
            )
            THEN RAISE(ABORT, 'stat_modifier already linked to an item or level')
        END;
END;

CREATE TRIGGER IF NOT EXISTS trg_stat_modifier_link_exclusive_item_update
BEFORE UPDATE ON item_stat_modifiers
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN EXISTS (
                SELECT 1 FROM unit_stat_modifiers usm WHERE usm.stat_modifier_id = NEW.stat_modifier_id
            )
            OR EXISTS (
                SELECT 1 FROM level_stat_modifiers lsm WHERE lsm.stat_modifier_id = NEW.stat_modifier_id
            )
            THEN RAISE(ABORT, 'stat_modifier already linked to a unit or level')
        END;
END;

CREATE TRIGGER IF NOT EXISTS trg_stat_modifier_link_exclusive_level_update
BEFORE UPDATE ON level_stat_modifiers
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN EXISTS (
                SELECT 1 FROM unit_stat_modifiers usm WHERE usm.stat_modifier_id = NEW.stat_modifier_id
            )
            OR EXISTS (
                SELECT 1 FROM item_stat_modifiers ism WHERE ism.stat_modifier_id = NEW.stat_modifier_id
            )
            THEN RAISE(ABORT, 'stat_modifier already linked to a unit or item')
        END;
END;
