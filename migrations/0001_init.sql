-- Migration: initialize full schema (collapsed from prior incremental migrations).
--
-- This file is intended for a clean reset: it creates the complete "current" schema
-- in one shot.
--
-- Notes:
-- - SQLite is assumed.
-- - We avoid wrapping this migration in an explicit transaction because the migration
--   runner may already do so.
-- - We enable foreign key enforcement.
-- - We create tables/triggers/indexes idempotently with IF NOT EXISTS where possible.
-- - Current schema reflects:
--     - core cards: units, items, levels
--     - actions + attacks/interactions extensions
--     - optional action links to unit/item/level (multiple actions per card allowed)
--     - armor_modifiers + link tables
--     - hex_grids (with name) + hex_tiles (user_data_json dropped)

PRAGMA foreign_keys = ON;

----------------------------------------------------------------------
-- Core cards
----------------------------------------------------------------------

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

----------------------------------------------------------------------
-- Actions + extensions
----------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS actions (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    name               TEXT NOT NULL UNIQUE,
    action_point_cost  INTEGER NOT NULL,
    action_type        TEXT NOT NULL,
    text               TEXT NOT NULL,

    -- Optional association to exactly one of (unit,item,level) per action.
    -- Multiple actions may reference the same card (no UNIQUE constraints).
    unit_id             INTEGER NULL,
    item_id             INTEGER NULL,
    level_id            INTEGER NULL,

    created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (length(trim(name)) > 0),
    CHECK (length(trim(text)) > 0),
    CHECK (action_point_cost >= 0),
    CHECK (action_type IN ('Interaction', 'Attack'))
);

CREATE INDEX IF NOT EXISTS idx_actions_name ON actions(name);
CREATE INDEX IF NOT EXISTS idx_actions_unit_id ON actions(unit_id);
CREATE INDEX IF NOT EXISTS idx_actions_item_id ON actions(item_id);
CREATE INDEX IF NOT EXISTS idx_actions_level_id ON actions(level_id);

CREATE TRIGGER IF NOT EXISTS trg_actions_updated_at
AFTER UPDATE ON actions
FOR EACH ROW
BEGIN
    UPDATE actions
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;

-- Enforce: only one of unit_id/item_id/level_id may be set per action,
-- and referenced card must exist when set.
CREATE TRIGGER IF NOT EXISTS trg_actions_validate_action_links_insert
BEFORE INSERT ON actions
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN
                (NEW.unit_id IS NOT NULL AND (NEW.item_id IS NOT NULL OR NEW.level_id IS NOT NULL))
                OR (NEW.item_id IS NOT NULL AND NEW.level_id IS NOT NULL)
            THEN
                RAISE(ABORT, 'actions may be linked to at most one of unit_id, item_id, level_id')
        END;

    SELECT
        CASE
            WHEN NEW.unit_id IS NOT NULL
                 AND (SELECT id FROM units WHERE id = NEW.unit_id) IS NULL
            THEN RAISE(ABORT, 'actions.unit_id references missing units.id')
        END;

    SELECT
        CASE
            WHEN NEW.item_id IS NOT NULL
                 AND (SELECT id FROM items WHERE id = NEW.item_id) IS NULL
            THEN RAISE(ABORT, 'actions.item_id references missing items.id')
        END;

    SELECT
        CASE
            WHEN NEW.level_id IS NOT NULL
                 AND (SELECT id FROM levels WHERE id = NEW.level_id) IS NULL
            THEN RAISE(ABORT, 'actions.level_id references missing levels.id')
        END;
END;

CREATE TRIGGER IF NOT EXISTS trg_actions_validate_action_links_update
BEFORE UPDATE OF unit_id, item_id, level_id ON actions
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN
                (NEW.unit_id IS NOT NULL AND (NEW.item_id IS NOT NULL OR NEW.level_id IS NOT NULL))
                OR (NEW.item_id IS NOT NULL AND NEW.level_id IS NOT NULL)
            THEN
                RAISE(ABORT, 'actions may be linked to at most one of unit_id, item_id, level_id')
        END;

    SELECT
        CASE
            WHEN NEW.unit_id IS NOT NULL
                 AND (SELECT id FROM units WHERE id = NEW.unit_id) IS NULL
            THEN RAISE(ABORT, 'actions.unit_id references missing units.id')
        END;

    SELECT
        CASE
            WHEN NEW.item_id IS NOT NULL
                 AND (SELECT id FROM items WHERE id = NEW.item_id) IS NULL
            THEN RAISE(ABORT, 'actions.item_id references missing items.id')
        END;

    SELECT
        CASE
            WHEN NEW.level_id IS NOT NULL
                 AND (SELECT id FROM levels WHERE id = NEW.level_id) IS NULL
            THEN RAISE(ABORT, 'actions.level_id references missing levels.id')
        END;
END;

-- Emulate ON DELETE SET NULL for linked actions when a card is deleted.
CREATE TRIGGER IF NOT EXISTS trg_units_delete_clear_actions
AFTER DELETE ON units
FOR EACH ROW
BEGIN
    UPDATE actions SET unit_id = NULL WHERE unit_id = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_items_delete_clear_actions
AFTER DELETE ON items
FOR EACH ROW
BEGIN
    UPDATE actions SET item_id = NULL WHERE item_id = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_levels_delete_clear_actions
AFTER DELETE ON levels
FOR EACH ROW
BEGIN
    UPDATE actions SET level_id = NULL WHERE level_id = OLD.id;
END;

-- 1:1 extension tables
CREATE TABLE IF NOT EXISTS attacks (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    action_id    INTEGER NOT NULL UNIQUE,

    damage       INTEGER NOT NULL,
    damage_type  TEXT NOT NULL,
    skill        TEXT NOT NULL,
    target       INTEGER NOT NULL,
    range        INTEGER NOT NULL,

    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (action_id) REFERENCES actions(id) ON DELETE CASCADE,

    CHECK (damage >= 0),
    CHECK (range >= 0),
    CHECK (target BETWEEN 1 AND 14),
    CHECK (damage_type IN ('Arcane', 'Physical')),
    CHECK (skill IN ('Strength', 'Focus', 'Intelligence', 'Knowledge', 'Agility'))
);

CREATE INDEX IF NOT EXISTS idx_attacks_action_id ON attacks(action_id);

CREATE TRIGGER IF NOT EXISTS trg_attacks_updated_at
AFTER UPDATE ON attacks
FOR EACH ROW
BEGIN
    UPDATE attacks
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS interactions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    action_id  INTEGER NOT NULL UNIQUE,

    range      INTEGER NOT NULL,
    skill      TEXT NOT NULL,
    target     INTEGER NULL,

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (action_id) REFERENCES actions(id) ON DELETE CASCADE,

    CHECK (range >= 0),
    CHECK (target IS NULL OR target BETWEEN 1 AND 14),
    CHECK (skill IN ('Strength', 'Focus', 'Intelligence', 'Knowledge', 'Agility'))
);

CREATE INDEX IF NOT EXISTS idx_interactions_action_id ON interactions(action_id);

CREATE TRIGGER IF NOT EXISTS trg_interactions_updated_at
AFTER UPDATE ON interactions
FOR EACH ROW
BEGIN
    UPDATE interactions
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;

----------------------------------------------------------------------
-- Stat modifiers + links
----------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS stat_modifiers (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Which stat does this modifier apply to?
    stat          TEXT NOT NULL,

    -- Numeric magnitude
    value         INTEGER NOT NULL,

    -- Operator: Add or Subtract
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

-- Link tables: a stat modifier may be associated with at most one of (unit, item, level).
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

-- Enforce "associated with exactly one card type" across the three link tables.
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

----------------------------------------------------------------------
-- Armor modifiers + links
----------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS armor_modifiers (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,

    card_id       INTEGER NOT NULL,
    value         INTEGER NOT NULL,
    suit          TEXT NOT NULL,

    -- DamageType enumerator (Arcane or Physical)
    damage_type   TEXT NOT NULL,

    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (suit IN ('Spades', 'Clubs', 'Diamonds', 'Hearts')),
    CHECK (damage_type IN ('Arcane', 'Physical'))
);

CREATE INDEX IF NOT EXISTS idx_armor_modifiers_card_id ON armor_modifiers(card_id);
CREATE INDEX IF NOT EXISTS idx_armor_modifiers_suit ON armor_modifiers(suit);
CREATE INDEX IF NOT EXISTS idx_armor_modifiers_damage_type ON armor_modifiers(damage_type);

CREATE TRIGGER IF NOT EXISTS trg_armor_modifiers_updated_at
AFTER UPDATE ON armor_modifiers
FOR EACH ROW
BEGIN
    UPDATE armor_modifiers
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS item_armor_modifiers (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id            INTEGER NOT NULL,
    armor_modifier_id  INTEGER NOT NULL UNIQUE,

    created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
    FOREIGN KEY (armor_modifier_id) REFERENCES armor_modifiers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_item_armor_modifiers_item_id
ON item_armor_modifiers(item_id);

CREATE TABLE IF NOT EXISTS level_armor_modifiers (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    level_id           INTEGER NOT NULL,
    armor_modifier_id  INTEGER NOT NULL UNIQUE,

    created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (level_id) REFERENCES levels(id) ON DELETE CASCADE,
    FOREIGN KEY (armor_modifier_id) REFERENCES armor_modifiers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_level_armor_modifiers_level_id
ON level_armor_modifiers(level_id);

-- Enforce "associated with an Item, or Level card" (but not both).
CREATE TRIGGER IF NOT EXISTS trg_armor_modifier_link_exclusive_item
BEFORE INSERT ON item_armor_modifiers
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN EXISTS (
                SELECT 1
                FROM level_armor_modifiers lam
                WHERE lam.armor_modifier_id = NEW.armor_modifier_id
            )
            THEN RAISE(ABORT, 'armor_modifier already linked to a level')
        END;
END;

CREATE TRIGGER IF NOT EXISTS trg_armor_modifier_link_exclusive_level
BEFORE INSERT ON level_armor_modifiers
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN EXISTS (
                SELECT 1
                FROM item_armor_modifiers iam
                WHERE iam.armor_modifier_id = NEW.armor_modifier_id
            )
            THEN RAISE(ABORT, 'armor_modifier already linked to an item')
        END;
END;

CREATE TRIGGER IF NOT EXISTS trg_armor_modifier_link_exclusive_item_update
BEFORE UPDATE ON item_armor_modifiers
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN EXISTS (
                SELECT 1
                FROM level_armor_modifiers lam
                WHERE lam.armor_modifier_id = NEW.armor_modifier_id
            )
            THEN RAISE(ABORT, 'armor_modifier already linked to a level')
        END;
END;

CREATE TRIGGER IF NOT EXISTS trg_armor_modifier_link_exclusive_level_update
BEFORE UPDATE ON level_armor_modifiers
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN EXISTS (
                SELECT 1
                FROM item_armor_modifiers iam
                WHERE iam.armor_modifier_id = NEW.armor_modifier_id
            )
            THEN RAISE(ABORT, 'armor_modifier already linked to an item')
        END;
END;

----------------------------------------------------------------------
-- Hex grids + tiles (current schema: hex_grids has name; hex_tiles has no user_data_json)
----------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS hex_grids (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    width       INTEGER NOT NULL,
    height      INTEGER NOT NULL,

    -- Added in later migration; included here in current schema.
    name        TEXT NOT NULL,

    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (width > 0),
    CHECK (height > 0),
    CHECK (length(trim(name)) > 0)
);

CREATE TRIGGER IF NOT EXISTS trg_hex_grids_updated_at
AFTER UPDATE ON hex_grids
FOR EACH ROW
BEGIN
    UPDATE hex_grids
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;

-- Enforce uniqueness for non-empty names.
CREATE UNIQUE INDEX IF NOT EXISTS idx_hex_grids_name_unique_nonempty
ON hex_grids(name)
WHERE length(trim(name)) > 0;

CREATE INDEX IF NOT EXISTS idx_hex_grids_name
ON hex_grids(name);

CREATE TABLE IF NOT EXISTS hex_tiles (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    hex_grid_id INTEGER NOT NULL,

    x           INTEGER NOT NULL,
    y           INTEGER NOT NULL,

    -- Optional associations / metadata
    unit_id     INTEGER NULL,
    item_id     INTEGER NULL,
    level_id    INTEGER NULL,
    type        TEXT NULL,

    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (hex_grid_id) REFERENCES hex_grids(id) ON DELETE CASCADE,

    UNIQUE (hex_grid_id, x, y)
);

CREATE INDEX IF NOT EXISTS idx_hex_tiles_grid_id
ON hex_tiles(hex_grid_id);

CREATE INDEX IF NOT EXISTS idx_hex_tiles_grid_id_xy
ON hex_tiles(hex_grid_id, x, y);

CREATE TRIGGER IF NOT EXISTS trg_hex_tiles_updated_at
AFTER UPDATE ON hex_tiles
FOR EACH ROW
BEGIN
    UPDATE hex_tiles
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;
