//! Hex grid domain model + SQLite persistence helpers.
//!
//! This module is self-contained so you can unit test persistence against an in-memory SQLite DB.
//!
//! Schema assumptions (created by migration):
//! - `hex_grids(id, width, height, created_at, updated_at)`
//! - `hex_tiles(id, hex_grid_id, x, y, user_data_json, created_at, updated_at)`
//!
//! Notes on semantics:
//! - A "tile space" exists for every (x,y) within the grid bounds, but it may be empty.
//! - "Removing tile spaces" for shaping (e.g. cutting corners) is represented by setting that
//!   coordinate to empty (i.e. deleting any `hex_tiles` row for that coordinate). This keeps the
//!   persisted schema simple and matches the requirement that spaces can be empty or populated.
//!
//! Coordinate system:
//! - Uses integer (x,y) within [0..width) × [0..height).
//! - This module does not enforce axial/cube hex math; it simply stores a hex-tile grid container.
//!
//! If you later want to support true "holes that are not addressable", add a `hex_grid_voids` table
//! or a `present` flag per coordinate.

use anyhow::{Result, anyhow};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub x: i32,
    pub y: i32,
}

impl HexCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexTile {
    pub coord: HexCoord,
    /// Arbitrary user JSON payload stored as text. (Not validated here.)
    pub user_data_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexGrid {
    pub id: Option<i64>,
    pub name: String,
    pub width: i32,
    pub height: i32,
}

impl HexGrid {
    pub fn generate(name: impl Into<String>, width: i32, height: i32) -> Result<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(anyhow!("name must be non-empty"));
        }
        if width <= 0 || height <= 0 {
            return Err(anyhow!("width and height must be > 0"));
        }
        Ok(Self {
            id: None,
            name,
            width,
            height,
        })
    }

    pub fn contains(&self, c: HexCoord) -> bool {
        c.x >= 0 && c.y >= 0 && c.x < self.width && c.y < self.height
    }

    fn require_contains(&self, c: HexCoord) -> Result<()> {
        if !self.contains(c) {
            return Err(anyhow!(
                "coordinate out of bounds: ({},{}) not in 0..{} x 0..{}",
                c.x,
                c.y,
                self.width,
                self.height
            ));
        }
        Ok(())
    }

    /// Inserts the grid row, returning the new id and updating `self.id`.
    pub fn insert(&mut self, conn: &Connection) -> Result<i64> {
        conn.execute(
            r#"
            INSERT INTO hex_grids (name, width, height)
            VALUES (?1, ?2, ?3)
            "#,
            params![self.name, self.width, self.height],
        )?;

        let id = conn.last_insert_rowid();
        self.id = Some(id);
        Ok(id)
    }

    pub fn load(conn: &Connection, id: i64) -> Result<Self> {
        let row = conn
            .query_row(
                r#"
                SELECT id, name, width, height
                FROM hex_grids
                WHERE id = ?1
                "#,
                params![id],
                |r| {
                    Ok(HexGrid {
                        id: Some(r.get::<_, i64>(0)?),
                        name: r.get::<_, String>(1)?,
                        width: r.get::<_, i32>(2)?,
                        height: r.get::<_, i32>(3)?,
                    })
                },
            )
            .optional()?;

        row.ok_or_else(|| anyhow!("hex grid not found: id={}", id))
    }

    fn require_id(&self) -> Result<i64> {
        self.id
            .ok_or_else(|| anyhow!("hex grid has no id (insert it first)"))
    }

    /// Add/replace a tile at `coord`.
    ///
    /// - If `user_data_json` is `None`, this still creates a tile row (i.e. a populated tile with
    ///   no user data yet). If you want it empty, call `remove_tile`.
    pub fn put_tile(
        &self,
        conn: &Connection,
        coord: HexCoord,
        user_data_json: Option<&str>,
    ) -> Result<()> {
        self.require_contains(coord)?;
        let grid_id = self.require_id()?;

        conn.execute(
            r#"
            INSERT INTO hex_tiles (hex_grid_id, x, y, user_data_json)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(hex_grid_id, x, y)
            DO UPDATE SET
                user_data_json = excluded.user_data_json,
                updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            "#,
            params![grid_id, coord.x, coord.y, user_data_json],
        )?;
        Ok(())
    }

    /// Removes (empties) a tile space at `coord` by deleting any persisted tile row.
    pub fn remove_tile(&self, conn: &Connection, coord: HexCoord) -> Result<()> {
        self.require_contains(coord)?;
        let grid_id = self.require_id()?;

        conn.execute(
            r#"
            DELETE FROM hex_tiles
            WHERE hex_grid_id = ?1 AND x = ?2 AND y = ?3
            "#,
            params![grid_id, coord.x, coord.y],
        )?;
        Ok(())
    }

    pub fn get_tile(&self, conn: &Connection, coord: HexCoord) -> Result<Option<HexTile>> {
        self.require_contains(coord)?;
        let grid_id = self.require_id()?;

        let tile = conn
            .query_row(
                r#"
                SELECT x, y, user_data_json
                FROM hex_tiles
                WHERE hex_grid_id = ?1 AND x = ?2 AND y = ?3
                "#,
                params![grid_id, coord.x, coord.y],
                |r| {
                    Ok(HexTile {
                        coord: HexCoord::new(r.get::<_, i32>(0)?, r.get::<_, i32>(1)?),
                        user_data_json: r.get::<_, Option<String>>(2)?,
                    })
                },
            )
            .optional()?;

        Ok(tile)
    }

    pub fn list_tiles(&self, conn: &Connection) -> Result<Vec<HexTile>> {
        let grid_id = self.require_id()?;

        let mut stmt = conn.prepare(
            r#"
            SELECT x, y, user_data_json
            FROM hex_tiles
            WHERE hex_grid_id = ?1
            ORDER BY y ASC, x ASC
            "#,
        )?;

        let mut rows = stmt.query(params![grid_id])?;
        let mut out = Vec::new();
        while let Some(r) = rows.next()? {
            out.push(HexTile {
                coord: HexCoord::new(r.get::<_, i32>(0)?, r.get::<_, i32>(1)?),
                user_data_json: r.get::<_, Option<String>>(2)?,
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory sqlite db");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("enable foreign_keys");
        conn
    }

    fn create_hex_schema(conn: &Connection) {
        // Minimal schema matching the intended migration for hex grids/tiles.
        conn.execute_batch(
            r#"
            CREATE TABLE hex_grids (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL UNIQUE,
                width       INTEGER NOT NULL,
                height      INTEGER NOT NULL,

                created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

                CHECK (length(trim(name)) > 0),
                CHECK (width > 0),
                CHECK (height > 0)
            );

            CREATE INDEX idx_hex_grids_name ON hex_grids(name);

            CREATE TRIGGER trg_hex_grids_updated_at
            AFTER UPDATE ON hex_grids
            FOR EACH ROW
            BEGIN
                UPDATE hex_grids
                SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                WHERE id = OLD.id;
            END;

            CREATE TABLE hex_tiles (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                hex_grid_id    INTEGER NOT NULL,
                x              INTEGER NOT NULL,
                y              INTEGER NOT NULL,
                user_data_json TEXT NULL,

                created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

                FOREIGN KEY (hex_grid_id) REFERENCES hex_grids(id) ON DELETE CASCADE,

                -- Enforce single tile per coordinate.
                UNIQUE (hex_grid_id, x, y)
            );

            CREATE INDEX idx_hex_tiles_grid ON hex_tiles(hex_grid_id);
            CREATE INDEX idx_hex_tiles_grid_xy ON hex_tiles(hex_grid_id, x, y);

            CREATE TRIGGER trg_hex_tiles_updated_at
            AFTER UPDATE ON hex_tiles
            FOR EACH ROW
            BEGIN
                UPDATE hex_tiles
                SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                WHERE id = OLD.id;
            END;
            "#,
        )
        .expect("create hex schema");
    }

    fn cut_corners_to_hex_shape(grid: &HexGrid, conn: &Connection) -> Result<()> {
        // Example: create a "hex" silhouette within a rectangular bounding box using empty spaces.
        //
        // For the unit test, we want deterministic behavior: (0,0) should be empty after carving.
        // We'll use a simple "cut the four corners" rule by requiring each coordinate to be within
        // `r` of *each* edge in a combined way:
        //   dx = abs(x-cx), dy = abs(y-cy)
        //   keep if (dx + dy) <= r
        //
        // This yields a diamond in a rectangular grid, which is sufficient to demonstrate:
        // - generating a grid
        // - carving a silhouette by leaving empty spaces
        // - persisting populated tiles only
        let w = grid.width;
        let h = grid.height;
        let cx = w / 2;
        let cy = h / 2;
        let r = w.min(h) / 2;

        for y in 0..h {
            for x in 0..w {
                let dx = (x - cx).abs();
                let dy = (y - cy).abs();

                let keep = dx + dy <= r;
                let coord = HexCoord::new(x, y);

                if keep {
                    // Give each kept tile a tiny JSON payload.
                    let json = format!(r#"{{"x":{},"y":{}}}"#, x, y);
                    grid.put_tile(conn, coord, Some(&json))?;
                } else {
                    grid.remove_tile(conn, coord)?;
                }
            }
        }

        Ok(())
    }

    #[test]
    fn generate_grid_insert_and_roundtrip_tiles_in_sqlite() -> Result<()> {
        let conn = open_in_memory_db();
        create_hex_schema(&conn);

        let mut grid = HexGrid::generate("test-grid", 7, 7)?;
        let grid_id = grid.insert(&conn)?;

        // carve into a hex-ish silhouette using empty spaces
        cut_corners_to_hex_shape(&grid, &conn)?;

        // Persisted tiles should be queryable via a fresh loaded grid
        let loaded = HexGrid::load(&conn, grid_id)?;
        assert_eq!(loaded.name, "test-grid");
        assert_eq!(loaded.width, 7);
        assert_eq!(loaded.height, 7);

        // Verify some expected kept/removed coordinates.
        // Center should exist.
        let center = HexCoord::new(3, 3);
        let t = loaded.get_tile(&conn, center)?;
        assert!(t.is_some());
        assert_eq!(
            t.unwrap().user_data_json.as_deref(),
            Some(r#"{"x":3,"y":3}"#)
        );

        // A corner should be empty (removed).
        let corner = HexCoord::new(0, 0);
        let t = loaded.get_tile(&conn, corner)?;
        assert!(t.is_none());

        // Ensure list_tiles returns a non-zero count but less than full grid area
        let tiles = loaded.list_tiles(&conn)?;
        assert!(!tiles.is_empty());
        assert!(tiles.len() < (loaded.width * loaded.height) as usize);

        // Replace a tile payload and ensure it updates
        loaded.put_tile(&conn, center, Some(r#"{"hello":"world"}"#))?;
        let t2 = loaded.get_tile(&conn, center)?;
        assert_eq!(
            t2.unwrap().user_data_json.as_deref(),
            Some(r#"{"hello":"world"}"#)
        );

        // Remove a tile and ensure it disappears
        loaded.remove_tile(&conn, center)?;
        let t3 = loaded.get_tile(&conn, center)?;
        assert!(t3.is_none());

        Ok(())
    }

    #[test]
    fn bounds_checking_rejects_out_of_range_coordinates() -> Result<()> {
        let conn = open_in_memory_db();
        create_hex_schema(&conn);

        let mut grid = HexGrid::generate("bounds-grid", 3, 2)?;
        grid.insert(&conn)?;

        let err = grid
            .put_tile(&conn, HexCoord::new(3, 0), Some(r#"{}"#))
            .unwrap_err();
        assert!(err.to_string().contains("out of bounds"));

        let err = grid.remove_tile(&conn, HexCoord::new(0, 2)).unwrap_err();
        assert!(err.to_string().contains("out of bounds"));

        Ok(())
    }
}
