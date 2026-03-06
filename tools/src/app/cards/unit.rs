use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};

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
/// This updates stats for an existing unit with the same `name`.
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

/// Renames a unit (updates the primary key `name`) and updates all fields.
///
/// - `old_name` identifies the existing row.
/// - `card.name` is the new name.
/// - If `old_name == card.name`, this behaves like `update_card`.
///
/// Returns `Ok(None)` if no unit with `old_name` exists.
pub fn rename_and_update_card(
    conn: &Connection,
    old_name: &str,
    card: &Unit,
) -> Result<Option<Unit>> {
    let old_name = old_name.trim();
    if old_name.is_empty() {
        anyhow::bail!("old_name must be non-empty");
    }
    validate_card(card)?;

    if old_name == card.name.trim() {
        return update_card(conn, card);
    }

    let changed = conn
        .execute(
            r#"
            UPDATE units
            SET
                name = ?2,
                strength = ?3,
                focus = ?4,
                intelligence = ?5,
                agility = ?6,
                knowledge = ?7
            WHERE name = ?1
            "#,
            params![
                old_name,
                card.name,
                card.strength,
                card.focus,
                card.intelligence,
                card.agility,
                card.knowledge
            ],
        )
        .with_context(|| format!("Failed to rename unit `{}` to `{}`", old_name, card.name))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_card_persists_to_database() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let u = Unit {
            name: "Alice".to_string(),
            strength: 3,
            focus: 2,
            intelligence: 4,
            agility: 5,
            knowledge: 1,
        };

        let saved = save_card(&conn, &u).expect("save_card should succeed");
        assert_eq!(saved, u);

        let reloaded = get_card(&conn, "Alice")
            .expect("get_card should succeed")
            .expect("saved card should exist");
        assert_eq!(reloaded, u);
    }

    #[test]
    fn get_card_returns_none_for_missing() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let missing = get_card(&conn, "Missing").expect("get_card should succeed");
        assert!(missing.is_none());
    }

    #[test]
    fn update_card_persists_changes() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let u1 = Unit {
            name: "Alice".to_string(),
            strength: 3,
            focus: 2,
            intelligence: 4,
            agility: 5,
            knowledge: 1,
        };
        save_card(&conn, &u1).expect("save initial unit");

        let u2 = Unit {
            strength: 10,
            knowledge: 99,
            ..u1.clone()
        };

        let updated = update_card(&conn, &u2)
            .expect("update_card should succeed")
            .expect("row should exist to update");
        assert_eq!(updated, u2);

        let reloaded = get_card(&conn, "Alice")
            .expect("get_card should succeed")
            .expect("card should still exist");
        assert_eq!(reloaded, u2);
    }

    #[test]
    fn rename_and_update_card_renames_primary_key_and_updates_fields() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let u1 = Unit {
            name: "Alice".to_string(),
            strength: 3,
            focus: 2,
            intelligence: 4,
            agility: 5,
            knowledge: 1,
        };
        save_card(&conn, &u1).expect("save initial unit");

        let u2 = Unit {
            name: "Alicia".to_string(),
            strength: 10,
            focus: 11,
            intelligence: 12,
            agility: 13,
            knowledge: 14,
        };

        let renamed = rename_and_update_card(&conn, "Alice", &u2)
            .expect("rename_and_update_card should succeed")
            .expect("row should exist to rename");
        assert_eq!(renamed, u2);

        let old = get_card(&conn, "Alice").expect("get old after rename");
        assert!(old.is_none());

        let new = get_card(&conn, "Alicia")
            .expect("get new after rename")
            .expect("renamed card should exist");
        assert_eq!(new, u2);
    }

    #[test]
    fn update_card_returns_none_if_missing() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let u = Unit {
            name: "Nope".to_string(),
            strength: 1,
            focus: 1,
            intelligence: 1,
            agility: 1,
            knowledge: 1,
        };

        let updated = update_card(&conn, &u).expect("update_card should succeed");
        assert!(updated.is_none());
    }

    #[test]
    fn delete_card_removes_row() {
        let conn = crate::app::cards::test_support::open_in_memory_db();
        crate::app::cards::test_support::create_schema(&conn);

        let u = Unit {
            name: "Alice".to_string(),
            strength: 3,
            focus: 2,
            intelligence: 4,
            agility: 5,
            knowledge: 1,
        };
        save_card(&conn, &u).expect("save unit");

        let deleted = delete_card(&conn, "Alice").expect("delete_card should succeed");
        assert!(deleted);

        let after = get_card(&conn, "Alice").expect("get_card after delete");
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
