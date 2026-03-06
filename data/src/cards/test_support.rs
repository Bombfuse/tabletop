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

            -- Optional association: an action can belong to at most one of:
            -- - a unit, item, or level (enforced by triggers below)
            unit_id            INTEGER NULL,
            item_id            INTEGER NULL,
            level_id           INTEGER NULL,

            created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

            CHECK (length(trim(name)) > 0),
            CHECK (length(trim(text)) > 0),
            CHECK (action_point_cost >= 0),
            CHECK (action_type IN ('Interaction', 'Attack'))
        );

        -- NOTE: We intentionally do NOT enforce "one action per card".
        -- Multiple actions may be associated with the same unit/item/level.
        --
        -- Validate links and enforce "at most one of unit_id/item_id/level_id".
        CREATE TRIGGER trg_actions_validate_action_links_insert
        BEFORE INSERT ON actions
        FOR EACH ROW
        BEGIN
            SELECT
                CASE
                    WHEN
                        (NEW.unit_id IS NOT NULL AND (NEW.item_id IS NOT NULL OR NEW.level_id IS NOT NULL))
                        OR (NEW.item_id IS NOT NULL AND NEW.level_id IS NOT NULL)
                    THEN
                        RAISE(ABORT, 'actions may be linked to at most one of unit_id, item_id, level_id')
                END;

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

        CREATE TRIGGER trg_actions_validate_action_links_update
        BEFORE UPDATE OF unit_id, item_id, level_id ON actions
        FOR EACH ROW
        BEGIN
            SELECT
                CASE
                    WHEN
                        (NEW.unit_id IS NOT NULL AND (NEW.item_id IS NOT NULL OR NEW.level_id IS NOT NULL))
                        OR (NEW.item_id IS NOT NULL AND NEW.level_id IS NOT NULL)
                    THEN
                        RAISE(ABORT, 'actions may be linked to at most one of unit_id, item_id, level_id')
                END;

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

        -- Emulate ON DELETE SET NULL by clearing links when a card is deleted.
        CREATE TRIGGER trg_units_delete_clear_actions
        AFTER DELETE ON units
        FOR EACH ROW
        BEGIN
            UPDATE actions SET unit_id = NULL WHERE unit_id = OLD.id;
        END;

        CREATE TRIGGER trg_items_delete_clear_actions
        AFTER DELETE ON items
        FOR EACH ROW
        BEGIN
            UPDATE actions SET item_id = NULL WHERE item_id = OLD.id;
        END;

        CREATE TRIGGER trg_levels_delete_clear_actions
        AFTER DELETE ON levels
        FOR EACH ROW
        BEGIN
            UPDATE actions SET level_id = NULL WHERE level_id = OLD.id;
        END;
        "#,
    )
    .expect("create test schema");
}
