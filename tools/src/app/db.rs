use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

/// Opens (or creates) the SQLite database at `db_path` and applies basic PRAGMAs.
///
/// This delegates to the shared `data` crate so other binaries/libraries can reuse
/// the same connection setup logic.
pub fn open_db(db_path: &Path) -> Result<Connection> {
    data::db::open_db(db_path)
}

/// Ensures core schema exists (currently just the `migrations` table).
///
/// This delegates to the shared `data` crate so other binaries/libraries can reuse
/// the same initialization logic.
pub fn init_db(conn: &Connection) -> Result<()> {
    data::db::init_db(conn)
}
