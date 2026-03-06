-- Migration: allow multiple actions to be associated with a Unit/Item/Level
--
-- Prior behavior (from 0004_action_links.sql):
-- - actions.unit_id / actions.item_id / actions.level_id are nullable association columns.
-- - Partial UNIQUE indexes enforced "at most one action per unit/item/level":
--     uq_actions_unit_id, uq_actions_item_id, uq_actions_level_id
--
-- This migration drops those UNIQUE indexes so multiple actions may reference the same
-- unit_id/item_id/level_id.
--
-- Notes:
-- - We keep the (non-unique) lookup indexes (idx_actions_*_id) intact.
-- - We keep the "at most one of unit_id/item_id/level_id per action" validation triggers intact.
-- - SQLite supports DROP INDEX IF EXISTS.

PRAGMA foreign_keys = ON;

DROP INDEX IF EXISTS uq_actions_unit_id;
DROP INDEX IF EXISTS uq_actions_item_id;
DROP INDEX IF EXISTS uq_actions_level_id;

-- If you previously created any alternate UNIQUE indexes on these columns, drop them here as well.
-- (No-ops if they don't exist.)
DROP INDEX IF EXISTS unique_actions_unit_id;
DROP INDEX IF EXISTS unique_actions_item_id;
DROP INDEX IF EXISTS unique_actions_level_id;
