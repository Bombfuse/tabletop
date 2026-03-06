-- Migration: allow optionally associating Actions with a Level, Item, or Unit.
--
-- Design notes:
-- - We model this as three optional foreign keys on `actions`.
-- - Each action can be linked to at most one of: unit, item, or level.
-- - A given unit/item/level can have at most one linked action (enforced with UNIQUE).
-- - If the target card is deleted, the link is cleared (ON DELETE SET NULL).
-- - Uses partial unique indexes (SQLite) so multiple NULLs are allowed.
-- - Adds CHECK to ensure only one association is set.

PRAGMA foreign_keys = ON;

-- Add nullable association columns
ALTER TABLE actions ADD COLUMN unit_id  INTEGER NULL;
ALTER TABLE actions ADD COLUMN item_id  INTEGER NULL;
ALTER TABLE actions ADD COLUMN level_id INTEGER NULL;

-- Enforce "at most one association" (unit/item/level) per action.
-- (SQLite allows adding a CHECK via a table rebuild only, so we enforce via triggers below.)
--
-- However, we can still add indexes and FK constraints with triggers to validate.

-- Foreign key enforcement for new/updated rows (SQLite cannot add FK constraints via ALTER TABLE
-- without rebuilding; we emulate the safety with triggers).
CREATE TRIGGER IF NOT EXISTS trg_actions_validate_action_links_insert
BEFORE INSERT ON actions
FOR EACH ROW
BEGIN
    -- Only one of unit_id/item_id/level_id may be set.
    SELECT
        CASE
            WHEN
                (NEW.unit_id IS NOT NULL AND (NEW.item_id IS NOT NULL OR NEW.level_id IS NOT NULL))
                OR (NEW.item_id IS NOT NULL AND NEW.level_id IS NOT NULL)
            THEN
                RAISE(ABORT, 'actions may be linked to at most one of unit_id, item_id, level_id')
        END;

    -- Validate referenced row exists if set.
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
    -- Only one of unit_id/item_id/level_id may be set.
    SELECT
        CASE
            WHEN
                (NEW.unit_id IS NOT NULL AND (NEW.item_id IS NOT NULL OR NEW.level_id IS NOT NULL))
                OR (NEW.item_id IS NOT NULL AND NEW.level_id IS NOT NULL)
            THEN
                RAISE(ABORT, 'actions may be linked to at most one of unit_id, item_id, level_id')
        END;

    -- Validate referenced row exists if set.
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

-- Ensure "one action per card" for each card type (partial unique indexes allow many NULLs).
CREATE UNIQUE INDEX IF NOT EXISTS uq_actions_unit_id
ON actions(unit_id)
WHERE unit_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uq_actions_item_id
ON actions(item_id)
WHERE item_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uq_actions_level_id
ON actions(level_id)
WHERE level_id IS NOT NULL;

-- Helpful lookup indexes
CREATE INDEX IF NOT EXISTS idx_actions_unit_id ON actions(unit_id);
CREATE INDEX IF NOT EXISTS idx_actions_item_id ON actions(item_id);
CREATE INDEX IF NOT EXISTS idx_actions_level_id ON actions(level_id);

-- Emulate ON DELETE SET NULL by clearing links when a card is deleted.
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
