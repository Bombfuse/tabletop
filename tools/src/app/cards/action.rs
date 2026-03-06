use anyhow::Result;
use rusqlite::Connection;

pub use data::cards::action::{
    Action, ActionAssociation, ActionType, clear_association, get_association, set_association,
};

/// Lists actions ordered by name (ascending).
///
/// Delegates to the shared `data` crate.
pub fn list_cards(conn: &Connection) -> Result<Vec<Action>> {
    data::cards::action::list_cards(conn)
}

/// Inserts a new action.
///
/// Delegates to the shared `data` crate.
pub fn save_card(conn: &Connection, card: &Action) -> Result<Action> {
    data::cards::action::save_card(conn, card)
}

/// Updates an existing action (by name).
///
/// Delegates to the shared `data` crate.
pub fn update_card(conn: &Connection, card: &Action) -> Result<Option<Action>> {
    data::cards::action::update_card(conn, card)
}

/// Renames an action (updates the primary key `name`) and updates all fields.
///
/// Delegates to the shared `data` crate.
pub fn rename_and_update_card(
    conn: &Connection,
    old_name: &str,
    card: &Action,
) -> Result<Option<Action>> {
    data::cards::action::rename_and_update_card(conn, old_name, card)
}

/// Deletes an action by name.
///
/// Delegates to the shared `data` crate.
pub fn delete_card(conn: &Connection, name: &str) -> Result<bool> {
    data::cards::action::delete_card(conn, name)
}

/// Loads an action by exact name.
///
/// Delegates to the shared `data` crate.
pub fn get_card(conn: &Connection, name: &str) -> Result<Option<Action>> {
    data::cards::action::get_card(conn, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn open_in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory sqlite db");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("enable foreign_keys");
        conn
    }

    fn create_schema(conn: &Connection) {
        conn.execute_batch(
            r#"
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
        .expect("create schema");
    }

    fn insert_action(conn: &Connection, action: &Action) -> i64 {
        conn.execute(
            r#"
            INSERT INTO actions (name, action_point_cost, action_type, text)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            rusqlite::params![
                action.name,
                action.action_point_cost,
                match action.action_type {
                    ActionType::Interaction => "Interaction",
                    ActionType::Attack => "Attack",
                },
                action.text
            ],
        )
        .expect("insert action");

        conn.query_row(
            "SELECT id FROM actions WHERE name = ?1",
            rusqlite::params![action.name],
            |row| row.get(0),
        )
        .expect("load action id")
    }

    #[test]
    fn delete_action_cascades_to_attack() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let action = Action {
            name: "Strike".to_string(),
            action_point_cost: 1,
            action_type: ActionType::Attack,
            text: "Deal damage".to_string(),
        };
        let action_id = insert_action(&conn, &action);

        conn.execute(
            r#"
            INSERT INTO attacks (action_id, damage, damage_type, skill, target, range)
            VALUES (?1, 3, 'Physical', 'Strength', 10, 1)
            "#,
            rusqlite::params![action_id],
        )
        .expect("insert attack");

        let deleted = delete_card(&conn, "Strike").expect("delete action");
        assert!(deleted, "expected delete_card to delete the action row");

        let remaining_attacks: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM attacks WHERE action_id = ?1",
                rusqlite::params![action_id],
                |row| row.get(0),
            )
            .expect("count attacks");
        assert_eq!(
            remaining_attacks, 0,
            "attack row should be deleted via cascade"
        );
    }

    #[test]
    fn delete_action_cascades_to_interaction() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let action = Action {
            name: "Talk".to_string(),
            action_point_cost: 1,
            action_type: ActionType::Interaction,
            text: "Speak".to_string(),
        };
        let action_id = insert_action(&conn, &action);

        conn.execute(
            r#"
            INSERT INTO interactions (action_id, range, skill, target)
            VALUES (?1, 2, 'Knowledge', 9)
            "#,
            rusqlite::params![action_id],
        )
        .expect("insert interaction");

        let deleted = delete_card(&conn, "Talk").expect("delete action");
        assert!(deleted, "expected delete_card to delete the action row");

        let remaining_interactions: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM interactions WHERE action_id = ?1",
                rusqlite::params![action_id],
                |row| row.get(0),
            )
            .expect("count interactions");
        assert_eq!(
            remaining_interactions, 0,
            "interaction row should be deleted via cascade"
        );
    }
}
