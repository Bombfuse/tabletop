-- Migration: add "hex_grids" and "hex_tiles" tables.
--
-- A HexGrid is a container with width/height.
-- A tile space is empty unless a row exists in `hex_tiles` for a given (grid,x,y).
-- A tile optionally stores user JSON payload as text.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS hex_grids (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    width       INTEGER NOT NULL,
    height      INTEGER NOT NULL,

    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (width > 0),
    CHECK (height > 0)
);

CREATE TRIGGER IF NOT EXISTS trg_hex_grids_updated_at
AFTER UPDATE ON hex_grids
FOR EACH ROW
BEGIN
    UPDATE hex_grids
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS hex_tiles (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    hex_grid_id    INTEGER NOT NULL,

    -- Coordinates within the grid bounds; bounds are enforced in application code.
    x              INTEGER NOT NULL,
    y              INTEGER NOT NULL,

    -- Arbitrary user data JSON payload (stored as TEXT; not validated here).
    user_data_json TEXT NULL,

    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (hex_grid_id) REFERENCES hex_grids(id) ON DELETE CASCADE,

    -- Enforce: at most one tile per coordinate in a given grid.
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
