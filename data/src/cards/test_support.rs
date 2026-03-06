use rusqlite::Connection;

pub(crate) fn open_in_memory_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory sqlite db");
    conn.pragma_update(None, "foreign_keys", "ON")
        .expect("enable foreign_keys");
    conn
}

pub(crate) fn create_schema(conn: &Connection) {
    // Minimal schema matching the migrations, sufficient for unit tests.
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

        CREATE TABLE levels (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            name          TEXT NOT NULL UNIQUE,
            text          TEXT NOT NULL,

            created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

            CHECK (length(trim(name)) > 0),
            CHECK (length(trim(text)) > 0)
        );

        CREATE TABLE actions (
            id                 INTEGER PRIMARY KEY AUTOINCREMENT,
            name               TEXT NOT NULL UNIQUE,
            action_point_cost  INTEGER NOT NULL,
            action_type        TEXT NOT NULL,
            text               TEXT NOT NULL,

            created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

            CHECK (length(trim(name)) > 0),
            CHECK (length(trim(text)) > 0),
            CHECK (action_point_cost >= 0),
            CHECK (action_type IN ('Interaction', 'Attack'))
        );

        CREATE TABLE attacks (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            action_id    INTEGER NOT NULL UNIQUE,
            damage       INTEGER NOT NULL,
            damage_type  TEXT NOT NULL,
            skill        TEXT NOT NULL,
            target       INTEGER NOT NULL,
            range        INTEGER NOT NULL,

            created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

            FOREIGN KEY (action_id) REFERENCES actions(id) ON DELETE CASCADE,

            CHECK (damage >= 0),
            CHECK (range >= 0),
            CHECK (target BETWEEN 1 AND 14),
            CHECK (damage_type IN ('Arcane', 'Physical')),
            CHECK (skill IN ('Strength', 'Focus', 'Intelligence', 'Knowledge', 'Agility'))
        );

        CREATE TABLE interactions (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            action_id  INTEGER NOT NULL UNIQUE,
            range      INTEGER NOT NULL,
            skill      TEXT NOT NULL,
            target     INTEGER NULL,

            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

            FOREIGN KEY (action_id) REFERENCES actions(id) ON DELETE CASCADE,

            CHECK (range >= 0),
            CHECK (target IS NULL OR target BETWEEN 1 AND 14),
            CHECK (skill IN ('Strength', 'Focus', 'Intelligence', 'Knowledge', 'Agility'))
        );
        "#,
    )
    .expect("create test schema");
}
