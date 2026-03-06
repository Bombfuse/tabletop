-- Migration: add armor modifiers and association links to items and levels.
--
-- Armor Modifier model:
-- - card_id: logical owner card id (from the core cards table, used by tooling)
-- - value: integer modifier value
-- - suit: one of Spades, Clubs, Diamonds, Hearts
--
-- Associations:
-- - An armor modifier can be linked to an item OR a level (optionally neither while drafting).
-- - Multiple armor modifiers per item/level are allowed.
-- - A modifier must not be linked to both an item and a level at the same time.
--
-- Notes:
-- - This DB uses SQLite.
-- - Timestamp triggers follow the existing convention in prior migrations.

CREATE TABLE IF NOT EXISTS armor_modifiers (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,

    -- "CardId" from the model request.
    -- This is intentionally generic: the UI layer can treat it as the parent card id.
    card_id       INTEGER NOT NULL,

    -- "Value" from the model request.
    value         INTEGER NOT NULL,

    -- "Suit" from the model request.
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

-- Link table: armor_modifiers -> items (many-to-one per modifier, many modifiers per item).
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

-- Link table: armor_modifiers -> levels (many-to-one per modifier, many modifiers per level).
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

-- Also protect UPDATEs that attempt to move a modifier into a conflicting link table.
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
