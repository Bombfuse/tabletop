use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};

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
/// "touch" semantics (ensure it exists) without changing the name.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_card_persists_to_database() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let it = Item {
            name: "Potion".to_string(),
        };

        let saved = save_card(&conn, &it).expect("save_card should succeed");
        assert_eq!(saved, it);

        let reloaded = get_card(&conn, "Potion")
            .expect("get_card should succeed")
            .expect("saved card should exist");
        assert_eq!(reloaded, it);
    }

    #[test]
    fn get_card_returns_none_for_missing() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let missing = get_card(&conn, "Missing").expect("get_card should succeed");
        assert!(missing.is_none());
    }

    #[test]
    fn update_card_returns_some_if_exists() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let it = Item {
            name: "Potion".to_string(),
        };
        save_card(&conn, &it).expect("save item");

        // Update is "touch" semantics for now, but it should still succeed and persist existence.
        let updated = update_card(&conn, &it)
            .expect("update_card should succeed")
            .expect("row should exist");
        assert_eq!(updated, it);

        let reloaded = get_card(&conn, "Potion")
            .expect("get_card should succeed")
            .expect("card should exist");
        assert_eq!(reloaded, it);
    }

    #[test]
    fn update_card_returns_none_if_missing() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let it = Item {
            name: "Nope".to_string(),
        };

        let updated = update_card(&conn, &it).expect("update_card should succeed");
        assert!(updated.is_none());
    }

    #[test]
    fn delete_card_removes_row() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let it = Item {
            name: "Potion".to_string(),
        };
        save_card(&conn, &it).expect("save item");

        let deleted = delete_card(&conn, "Potion").expect("delete_card should succeed");
        assert!(deleted);

        let after = get_card(&conn, "Potion").expect("get_card after delete");
        assert!(after.is_none());
    }

    #[test]
    fn delete_card_returns_false_if_missing() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let deleted = delete_card(&conn, "Missing").expect("delete_card should succeed");
        assert!(!deleted);
    }
}
