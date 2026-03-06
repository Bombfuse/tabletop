pub mod item;
pub mod unit;

#[cfg(test)]
pub(crate) mod test_support {
    use rusqlite::Connection;

    pub(crate) fn open_in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory sqlite db");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("enable foreign_keys");
        conn
    }

    pub(crate) fn create_schema(conn: &Connection) {
        // Minimal schema matching the migration, sufficient for unit tests.
        conn.execute_batch(
            r#"
            CREATE TABLE units (
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

            CREATE TABLE items (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                name          TEXT NOT NULL UNIQUE,

                created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

                CHECK (length(trim(name)) > 0)
            );
            "#,
        )
        .expect("create test schema");
    }
}
