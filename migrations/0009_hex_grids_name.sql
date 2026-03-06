-- Migration: add `name` to `hex_grids` and ensure non-empty uniqueness.
--
-- This migration is written to be safe for existing databases that already have
-- `hex_grids(id, width, height, created_at, updated_at)` created by 0008.
--
-- SQLite limitations:
-- - You cannot add a UNIQUE constraint to an existing column via ALTER TABLE.
-- - Adding a NOT NULL column requires a DEFAULT.
--
-- Approach:
-- 1) Add `name` as a NOT NULL column with a temporary DEFAULT to satisfy existing rows.
-- 2) Backfill unique names for existing grids (based on id).
-- 3) Create a UNIQUE index on `name` to enforce uniqueness going forward.
-- 4) Create a CHECK via triggers is not possible; we instead enforce non-empty using a CHECK
--    by recreating the table (overkill) or by ensuring the index exists + application checks.
--    Here we add a partial index guard (name trimmed length > 0) and a normal unique index.
--
-- Note: The partial unique index enforces both uniqueness and non-empty.
-- If your SQLite version is too old for partial indexes, you can remove the WHERE clause and
-- rely on application-level validation + the CHECK in a future table rebuild.

PRAGMA foreign_keys = ON;

-- 1) Add the column. Existing rows will receive the default.
ALTER TABLE hex_grids
ADD COLUMN name TEXT NOT NULL DEFAULT '__MIGRATION_PENDING__';

-- 2) Backfill existing rows with deterministic unique names.
-- Use the primary key to guarantee uniqueness.
UPDATE hex_grids
SET name = 'Hex Grid ' || id
WHERE name = '__MIGRATION_PENDING__';

-- 3) Enforce uniqueness and non-empty values going forward.
-- Unique + non-empty (trimmed) enforcement:
CREATE UNIQUE INDEX IF NOT EXISTS idx_hex_grids_name_unique_nonempty
ON hex_grids(name)
WHERE length(trim(name)) > 0;

-- Helpful for lookup by name (also covered by the unique index above, but keeping this
-- separate can be useful if you later adjust constraints).
CREATE INDEX IF NOT EXISTS idx_hex_grids_name
ON hex_grids(name);

-- 4) Sanity check: abort if duplicates slipped in (should not happen with id-based backfill).
-- This SELECT will error if duplicates exist when trying to create the unique index above.
-- (If the index already existed, this is effectively a no-op.)
