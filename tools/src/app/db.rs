use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

/// Opens (or creates) the SQLite database at `db_path` and applies basic PRAGMAs.
///
/// Notes:
/// - `journal_mode = WAL` is a persistent setting (stored in the DB file).
/// - `foreign_keys = ON` is per-connection and must be set each time.
pub fn open_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // Reasonable defaults for application DBs.
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;

    Ok(conn)
}

/// Ensures core schema exists (currently just the `migrations` table).
///
/// The `migrations` table records which migration filenames have been applied.
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS migrations (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            filename      TEXT NOT NULL UNIQUE,
            applied_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );
        "#,
    )?;

    Ok(())
}
