-- Migration: add `damage_type` to `armor_modifiers`.
--
-- Reason:
-- Older databases may have been created before `armor_modifiers.damage_type` existed.
-- Newer code expects this column to exist.
--
-- Notes:
-- - SQLite supports `ALTER TABLE ... ADD COLUMN`, but not `ADD COLUMN IF NOT EXISTS`.
-- - This migration is intended to run exactly once on databases missing the column.
-- - We use a DEFAULT so existing rows get a value.
-- - We add an index for query performance.

ALTER TABLE armor_modifiers
ADD COLUMN damage_type TEXT NOT NULL DEFAULT 'Physical';

-- Index for filtering/grouping by damage type.
CREATE INDEX IF NOT EXISTS idx_armor_modifiers_damage_type
ON armor_modifiers(damage_type);
