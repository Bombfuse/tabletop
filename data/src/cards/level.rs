use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level {
    pub name: String,
    pub text: String,
}

/// Lists levels ordered by name (ascending).
pub fn list_cards(conn: &Connection) -> Result<Vec<Level>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT name, text
            FROM levels
            ORDER BY name ASC
            "#,
        )
        .with_context(|| "Failed to prepare list levels query")?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Level {
                name: row.get(0)?,
                text: row.get(1)?,
            })
        })
        .with_context(|| "Failed to query levels")?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.with_context(|| "Failed to read level row")?);
    }

    Ok(out)
}

/// Inserts a new level.
///
/// If a level with the same name already exists, this will return an error
/// (because `save_card` is intentionally "create" semantics).
pub fn save_card(conn: &Connection, card: &Level) -> Result<Level> {
    validate_card(card)?;

    conn.execute(
        r#"
        INSERT INTO levels (name, text)
        VALUES (?1, ?2)
        "#,
        params![card.name, card.text],
    )
    .with_context(|| format!("Failed to save level `{}`", card.name))?;

    get_card(conn, &card.name)?
        .with_context(|| format!("Level `{}` was saved but could not be reloaded", card.name))
}

/// Updates an existing level (by name).
///
/// Returns `Ok(None)` if no level with that name exists.
pub fn update_card(conn: &Connection, card: &Level) -> Result<Option<Level>> {
    validate_card(card)?;

    let changed = conn
        .execute(
            r#"
            UPDATE levels
            SET text = ?2
            WHERE name = ?1
            "#,
            params![card.name, card.text],
        )
        .with_context(|| format!("Failed to update level `{}`", card.name))?;

    if changed == 0 {
        return Ok(None);
    }

    get_card(conn, &card.name)
}

/// Renames a level (updates the primary key `name`) and updates its text.
///
/// - `old_name` identifies the existing row.
/// - `card.name` is the new name.
/// - If `old_name == card.name`, this behaves like `update_card`.
///
/// Returns `Ok(None)` if no level with `old_name` exists.
pub fn rename_card(conn: &Connection, old_name: &str, card: &Level) -> Result<Option<Level>> {
    let old_name = crate::shared::require_non_empty_trimmed("old_name", old_name)?;
    validate_card(card)?;

    if old_name == card.name.trim() {
        return update_card(conn, card);
    }

    let changed = conn
        .execute(
            r#"
            UPDATE levels
            SET name = ?2,
                text = ?3
            WHERE name = ?1
            "#,
            params![old_name, card.name, card.text],
        )
        .with_context(|| format!("Failed to rename level `{}` to `{}`", old_name, card.name))?;

    if changed == 0 {
        return Ok(None);
    }

    get_card(conn, &card.name)
}

/// Deletes a level by name.
///
/// Returns `Ok(true)` if a row was deleted, `Ok(false)` if nothing matched.
pub fn delete_card(conn: &Connection, name: &str) -> Result<bool> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(false);
    }

    let changed = conn
        .execute("DELETE FROM levels WHERE name = ?1", params![name])
        .with_context(|| format!("Failed to delete level `{}`", name))?;
    Ok(changed > 0)
}

/// Loads a level by exact name.
///
/// Returns `Ok(None)` if not found.
pub fn get_card(conn: &Connection, name: &str) -> Result<Option<Level>> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(None);
    }

    conn.query_row(
        r#"
        SELECT name, text
        FROM levels
        WHERE name = ?1
        "#,
        params![name],
        |row| {
            Ok(Level {
                name: row.get(0)?,
                text: row.get(1)?,
            })
        },
    )
    .optional()
    .with_context(|| format!("Failed to fetch level `{}`", name))
}

fn validate_card(card: &Level) -> Result<()> {
    crate::shared::require_non_empty_trimmed("Level.name", &card.name)?;
    crate::shared::require_non_empty_trimmed("Level.text", &card.text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_card_persists_to_database() {
        let conn = crate::cards::test_support::open_in_memory_db();
        crate::cards::test_support::create_schema(&conn);

        let lv = Level {
            name: "Dungeon 1".to_string(),
            text: "A damp corridor lit by torches.".to_string(),
        };

        let saved = save_card(&conn, &lv).expect("save_card should succeed");
        assert_eq!(saved, lv);

        let reloaded = get_card(&conn, "Dungeon 1")
            .expect("get_card should succeed")
            .expect("saved card should exist");
        assert_eq!(reloaded, lv);
    }

    #[test]
    fn get_card_returns_none_for_missing() {
        let conn = crate::cards::test_support::open_in_memory_db();
        crate::cards::test_support::create_schema(&conn);

        let missing = get_card(&conn, "Missing").expect("get_card should succeed");
        assert!(missing.is_none());
    }

    #[test]
    fn update_card_updates_text_if_exists() {
        let conn = crate::cards::test_support::open_in_memory_db();
        crate::cards::test_support::create_schema(&conn);

        let lv = Level {
            name: "Dungeon 1".to_string(),
            text: "Old text".to_string(),
        };
        save_card(&conn, &lv).expect("save level");

        let updated_lv = Level {
            name: "Dungeon 1".to_string(),
            text: "New text".to_string(),
        };

        let updated = update_card(&conn, &updated_lv)
            .expect("update_card should succeed")
            .expect("row should exist");
        assert_eq!(updated, updated_lv);

        let reloaded = get_card(&conn, "Dungeon 1")
            .expect("get_card should succeed")
            .expect("card should exist");
        assert_eq!(reloaded, updated_lv);
    }

    #[test]
    fn rename_card_renames_primary_key_and_updates_text() {
        let conn = crate::cards::test_support::open_in_memory_db();
        crate::cards::test_support::create_schema(&conn);

        let lv1 = Level {
            name: "Dungeon 1".to_string(),
            text: "Text 1".to_string(),
        };
        save_card(&conn, &lv1).expect("save level");

        let lv2 = Level {
            name: "Dungeon 2".to_string(),
            text: "Text 2".to_string(),
        };

        let renamed = rename_card(&conn, "Dungeon 1", &lv2)
            .expect("rename_card should succeed")
            .expect("row should exist to rename");
        assert_eq!(renamed, lv2);

        let old = get_card(&conn, "Dungeon 1").expect("get old after rename");
        assert!(old.is_none());

        let new = get_card(&conn, "Dungeon 2")
            .expect("get new after rename")
            .expect("renamed card should exist");
        assert_eq!(new, lv2);
    }

    #[test]
    fn update_card_returns_none_if_missing() {
        let conn = crate::cards::test_support::open_in_memory_db();
        crate::cards::test_support::create_schema(&conn);

        let lv = Level {
            name: "Nope".to_string(),
            text: "Does not exist".to_string(),
        };

        let updated = update_card(&conn, &lv).expect("update_card should succeed");
        assert!(updated.is_none());
    }

    #[test]
    fn delete_card_removes_row() {
        let conn = crate::cards::test_support::open_in_memory_db();
        crate::cards::test_support::create_schema(&conn);

        let lv = Level {
            name: "Dungeon 1".to_string(),
            text: "Text".to_string(),
        };
        save_card(&conn, &lv).expect("save level");

        let deleted = delete_card(&conn, "Dungeon 1").expect("delete_card should succeed");
        assert!(deleted);

        let after = get_card(&conn, "Dungeon 1").expect("get_card after delete");
        assert!(after.is_none());
    }

    #[test]
    fn delete_card_returns_false_if_missing() {
        let conn = crate::cards::test_support::open_in_memory_db();
        crate::cards::test_support::create_schema(&conn);

        let deleted = delete_card(&conn, "Missing").expect("delete_card should succeed");
        assert!(!deleted);
    }
}
