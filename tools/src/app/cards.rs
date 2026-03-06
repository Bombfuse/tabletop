use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};

/// Card models + SQLite repository helpers.
///
/// This module is organized into per-card submodules:
/// - `unit`
/// - `item`
///
/// Each submodule exposes the following methods:
/// - `save_card`
/// - `delete_card`
/// - `update_card`
/// - `get_card`
///
/// Conventions:
/// - `name` is treated as a natural key (UNIQUE) so we can use it for retrieval and upsert-like behavior.
/// - Stats are stored as INTEGER in SQLite and modeled as `i64` here.

pub mod unit {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Unit {
        pub name: String,
        pub strength: i64,
        pub focus: i64,
        pub intelligence: i64,
        pub agility: i64,
        pub knowledge: i64,
    }

    /// Inserts a new unit.
    ///
    /// If a unit with the same name already exists, this will return an error
    /// (because `save_card` is intentionally "create" semantics).
    pub fn save_card(conn: &Connection, card: &Unit) -> Result<Unit> {
        validate_card(card)?;

        conn.execute(
            r#"
            INSERT INTO units (name, strength, focus, intelligence, agility, knowledge)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                card.name,
                card.strength,
                card.focus,
                card.intelligence,
                card.agility,
                card.knowledge
            ],
        )
        .with_context(|| format!("Failed to save unit `{}`", card.name))?;

        get_card(conn, &card.name)?
            .with_context(|| format!("Unit `{}` was saved but could not be reloaded", card.name))
    }

    /// Updates an existing unit (by name).
    ///
    /// Returns `Ok(None)` if no unit with that name exists.
    pub fn update_card(conn: &Connection, card: &Unit) -> Result<Option<Unit>> {
        validate_card(card)?;

        let changed = conn
            .execute(
                r#"
                UPDATE units
                SET
                    strength = ?2,
                    focus = ?3,
                    intelligence = ?4,
                    agility = ?5,
                    knowledge = ?6
                WHERE name = ?1
                "#,
                params![
                    card.name,
                    card.strength,
                    card.focus,
                    card.intelligence,
                    card.agility,
                    card.knowledge
                ],
            )
            .with_context(|| format!("Failed to update unit `{}`", card.name))?;

        if changed == 0 {
            return Ok(None);
        }

        get_card(conn, &card.name)
    }

    /// Deletes a unit by name.
    ///
    /// Returns `Ok(true)` if a row was deleted, `Ok(false)` if nothing matched.
    pub fn delete_card(conn: &Connection, name: &str) -> Result<bool> {
        let name = name.trim();
        if name.is_empty() {
            return Ok(false);
        }

        let changed = conn
            .execute("DELETE FROM units WHERE name = ?1", params![name])
            .with_context(|| format!("Failed to delete unit `{}`", name))?;
        Ok(changed > 0)
    }

    /// Loads a unit by exact name.
    ///
    /// Returns `Ok(None)` if not found.
    pub fn get_card(conn: &Connection, name: &str) -> Result<Option<Unit>> {
        let name = name.trim();
        if name.is_empty() {
            return Ok(None);
        }

        conn.query_row(
            r#"
            SELECT name, strength, focus, intelligence, agility, knowledge
            FROM units
            WHERE name = ?1
            "#,
            params![name],
            |row| {
                Ok(Unit {
                    name: row.get(0)?,
                    strength: row.get(1)?,
                    focus: row.get(2)?,
                    intelligence: row.get(3)?,
                    agility: row.get(4)?,
                    knowledge: row.get(5)?,
                })
            },
        )
        .optional()
        .with_context(|| format!("Failed to fetch unit `{}`", name))
    }

    fn validate_card(card: &Unit) -> Result<()> {
        if card.name.trim().is_empty() {
            anyhow::bail!("Unit.name must be non-empty");
        }
        Ok(())
    }
}

pub mod item {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Item {
        pub name: String,
    }

    /// Inserts a new item.
    ///
    /// If an item with the same name already exists, this will return an error
    /// (because `save_card` is intentionally "create" semantics).
    pub fn save_card(conn: &Connection, card: &Item) -> Result<Item> {
        validate_card(card)?;

        conn.execute(
            r#"
            INSERT INTO items (name)
            VALUES (?1)
            "#,
            params![card.name],
        )
        .with_context(|| format!("Failed to save item `{}`", card.name))?;

        get_card(conn, &card.name)?
            .with_context(|| format!("Item `{}` was saved but could not be reloaded", card.name))
    }

    /// Updates an existing item (by name).
    ///
    /// For the current model, the only field is `name`, so "update" is treated as
    /// "touch" (ensure it exists) without changing the name.
    ///
    /// Returns `Ok(None)` if no item with that name exists.
    pub fn update_card(conn: &Connection, card: &Item) -> Result<Option<Item>> {
        validate_card(card)?;

        let changed = conn
            .execute(
                r#"
                UPDATE items
                SET name = name
                WHERE name = ?1
                "#,
                params![card.name],
            )
            .with_context(|| format!("Failed to update item `{}`", card.name))?;

        if changed == 0 {
            return Ok(None);
        }

        get_card(conn, &card.name)
    }

    /// Deletes an item by name.
    ///
    /// Returns `Ok(true)` if a row was deleted, `Ok(false)` if nothing matched.
    pub fn delete_card(conn: &Connection, name: &str) -> Result<bool> {
        let name = name.trim();
        if name.is_empty() {
            return Ok(false);
        }

        let changed = conn
            .execute("DELETE FROM items WHERE name = ?1", params![name])
            .with_context(|| format!("Failed to delete item `{}`", name))?;
        Ok(changed > 0)
    }

    /// Loads an item by exact name.
    ///
    /// Returns `Ok(None)` if not found.
    pub fn get_card(conn: &Connection, name: &str) -> Result<Option<Item>> {
        let name = name.trim();
        if name.is_empty() {
            return Ok(None);
        }

        conn.query_row(
            r#"
            SELECT name
            FROM items
            WHERE name = ?1
            "#,
            params![name],
            |row| Ok(Item { name: row.get(0)? }),
        )
        .optional()
        .with_context(|| format!("Failed to fetch item `{}`", name))
    }

    fn validate_card(card: &Item) -> Result<()> {
        if card.name.trim().is_empty() {
            anyhow::bail!("Item.name must be non-empty");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{item, unit};
    use rusqlite::Connection;

    fn open_in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory sqlite db");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("enable foreign_keys");
        conn
    }

    fn create_schema(conn: &Connection) {
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

    // -----------------------
    // Unit tests (units table)
    // -----------------------

    #[test]
    fn unit_save_card_persists_to_database() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let u = unit::Unit {
            name: "Alice".to_string(),
            strength: 3,
            focus: 2,
            intelligence: 4,
            agility: 5,
            knowledge: 1,
        };

        let saved = unit::save_card(&conn, &u).expect("save_card should succeed");
        assert_eq!(saved, u);

        let reloaded = unit::get_card(&conn, "Alice")
            .expect("get_card should succeed")
            .expect("saved card should exist");
        assert_eq!(reloaded, u);
    }

    #[test]
    fn unit_get_card_returns_none_for_missing() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let missing =
            unit::get_card(&conn, "Missing").expect("get_card should succeed for missing row");
        assert!(missing.is_none());
    }

    #[test]
    fn unit_update_card_persists_changes() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let u1 = unit::Unit {
            name: "Alice".to_string(),
            strength: 3,
            focus: 2,
            intelligence: 4,
            agility: 5,
            knowledge: 1,
        };
        unit::save_card(&conn, &u1).expect("save initial unit");

        let u2 = unit::Unit {
            strength: 10,
            knowledge: 99,
            ..u1.clone()
        };

        let updated = unit::update_card(&conn, &u2)
            .expect("update_card should succeed")
            .expect("row should exist to update");
        assert_eq!(updated, u2);

        let reloaded = unit::get_card(&conn, "Alice")
            .expect("get_card should succeed")
            .expect("card should still exist");
        assert_eq!(reloaded, u2);
    }

    #[test]
    fn unit_update_card_returns_none_if_missing() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let u = unit::Unit {
            name: "Nope".to_string(),
            strength: 1,
            focus: 1,
            intelligence: 1,
            agility: 1,
            knowledge: 1,
        };

        let updated = unit::update_card(&conn, &u).expect("update_card should succeed");
        assert!(updated.is_none());
    }

    #[test]
    fn unit_delete_card_removes_row() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let u = unit::Unit {
            name: "Alice".to_string(),
            strength: 3,
            focus: 2,
            intelligence: 4,
            agility: 5,
            knowledge: 1,
        };
        unit::save_card(&conn, &u).expect("save unit");

        let deleted = unit::delete_card(&conn, "Alice").expect("delete_card should succeed");
        assert!(deleted);

        let after = unit::get_card(&conn, "Alice").expect("get_card after delete");
        assert!(after.is_none());
    }

    #[test]
    fn unit_delete_card_returns_false_if_missing() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let deleted = unit::delete_card(&conn, "Missing").expect("delete_card should succeed");
        assert!(!deleted);
    }

    // -----------------------
    // Item tests (items table)
    // -----------------------

    #[test]
    fn item_save_card_persists_to_database() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let it = item::Item {
            name: "Potion".to_string(),
        };

        let saved = item::save_card(&conn, &it).expect("save_card should succeed");
        assert_eq!(saved, it);

        let reloaded = item::get_card(&conn, "Potion")
            .expect("get_card should succeed")
            .expect("saved card should exist");
        assert_eq!(reloaded, it);
    }

    #[test]
    fn item_get_card_returns_none_for_missing() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let missing =
            item::get_card(&conn, "Missing").expect("get_card should succeed for missing row");
        assert!(missing.is_none());
    }

    #[test]
    fn item_update_card_returns_some_if_exists() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let it = item::Item {
            name: "Potion".to_string(),
        };
        item::save_card(&conn, &it).expect("save item");

        // Update is "touch" semantics for now, but it should still succeed and persist existence.
        let updated = item::update_card(&conn, &it)
            .expect("update_card should succeed")
            .expect("row should exist");
        assert_eq!(updated, it);

        let reloaded = item::get_card(&conn, "Potion")
            .expect("get_card should succeed")
            .expect("card should exist");
        assert_eq!(reloaded, it);
    }

    #[test]
    fn item_update_card_returns_none_if_missing() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let it = item::Item {
            name: "Nope".to_string(),
        };

        let updated = item::update_card(&conn, &it).expect("update_card should succeed");
        assert!(updated.is_none());
    }

    #[test]
    fn item_delete_card_removes_row() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let it = item::Item {
            name: "Potion".to_string(),
        };
        item::save_card(&conn, &it).expect("save item");

        let deleted = item::delete_card(&conn, "Potion").expect("delete_card should succeed");
        assert!(deleted);

        let after = item::get_card(&conn, "Potion").expect("get_card after delete");
        assert!(after.is_none());
    }

    #[test]
    fn item_delete_card_returns_false_if_missing() {
        let conn = open_in_memory_db();
        create_schema(&conn);

        let deleted = item::delete_card(&conn, "Missing").expect("delete_card should succeed");
        assert!(!deleted);
    }
}
