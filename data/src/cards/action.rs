use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Interaction,
    Attack,
}

impl ActionType {
    fn as_str(self) -> &'static str {
        match self {
            ActionType::Interaction => "Interaction",
            ActionType::Attack => "Attack",
        }
    }

    fn parse(s: &str) -> Result<Self> {
        match s {
            "Interaction" => Ok(ActionType::Interaction),
            "Attack" => Ok(ActionType::Attack),
            other => anyhow::bail!("Invalid ActionType `{}`", other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub name: String,
    pub action_point_cost: i64,
    pub action_type: ActionType,
    pub text: String,
}

/// Lists actions ordered by name (ascending).
pub fn list_cards(conn: &Connection) -> Result<Vec<Action>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT name, action_point_cost, action_type, text
            FROM actions
            ORDER BY name ASC
            "#,
        )
        .with_context(|| "Failed to prepare list actions query")?;

    let rows = stmt
        .query_map([], |row| {
            let action_type_str: String = row.get(2)?;
            let action_type = ActionType::parse(&action_type_str).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    2,
                    "action_type".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            Ok(Action {
                name: row.get(0)?,
                action_point_cost: row.get(1)?,
                action_type,
                text: row.get(3)?,
            })
        })
        .with_context(|| "Failed to query actions")?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.with_context(|| "Failed to read action row")?);
    }

    Ok(out)
}

/// Inserts a new action.
///
/// If an action with the same name already exists, this will return an error
/// (because `save_card` is intentionally "create" semantics).
pub fn save_card(conn: &Connection, card: &Action) -> Result<Action> {
    validate_card(card)?;

    conn.execute(
        r#"
        INSERT INTO actions (name, action_point_cost, action_type, text)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        params![
            card.name,
            card.action_point_cost,
            card.action_type.as_str(),
            card.text
        ],
    )
    .with_context(|| format!("Failed to save action `{}`", card.name))?;

    get_card(conn, &card.name)?
        .with_context(|| format!("Action `{}` was saved but could not be reloaded", card.name))
}

/// Updates an existing action (by name).
///
/// Returns `Ok(None)` if no action with that name exists.
pub fn update_card(conn: &Connection, card: &Action) -> Result<Option<Action>> {
    validate_card(card)?;

    let changed = conn
        .execute(
            r#"
            UPDATE actions
            SET
                action_point_cost = ?2,
                action_type = ?3,
                text = ?4
            WHERE name = ?1
            "#,
            params![
                card.name,
                card.action_point_cost,
                card.action_type.as_str(),
                card.text
            ],
        )
        .with_context(|| format!("Failed to update action `{}`", card.name))?;

    if changed == 0 {
        return Ok(None);
    }

    get_card(conn, &card.name)
}

/// Renames an action (updates the primary key `name`) and updates all fields.
///
/// - `old_name` identifies the existing row.
/// - `card.name` is the new name.
/// - If `old_name == card.name`, this behaves like `update_card`.
///
/// Returns `Ok(None)` if no action with `old_name` exists.
pub fn rename_and_update_card(
    conn: &Connection,
    old_name: &str,
    card: &Action,
) -> Result<Option<Action>> {
    let old_name = crate::shared::require_non_empty_trimmed("old_name", old_name)?;
    validate_card(card)?;

    if old_name == card.name.trim() {
        return update_card(conn, card);
    }

    let changed = conn
        .execute(
            r#"
            UPDATE actions
            SET
                name = ?2,
                action_point_cost = ?3,
                action_type = ?4,
                text = ?5
            WHERE name = ?1
            "#,
            params![
                old_name,
                card.name,
                card.action_point_cost,
                card.action_type.as_str(),
                card.text
            ],
        )
        .with_context(|| format!("Failed to rename action `{}` to `{}`", old_name, card.name))?;

    if changed == 0 {
        return Ok(None);
    }

    get_card(conn, &card.name)
}

/// Deletes an action by name.
///
/// Returns `Ok(true)` if a row was deleted, `Ok(false)` if nothing matched.
pub fn delete_card(conn: &Connection, name: &str) -> Result<bool> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(false);
    }

    let changed = conn
        .execute("DELETE FROM actions WHERE name = ?1", params![name])
        .with_context(|| format!("Failed to delete action `{}`", name))?;
    Ok(changed > 0)
}

/// Loads an action by exact name.
///
/// Returns `Ok(None)` if not found.
pub fn get_card(conn: &Connection, name: &str) -> Result<Option<Action>> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(None);
    }

    conn.query_row(
        r#"
        SELECT name, action_point_cost, action_type, text
        FROM actions
        WHERE name = ?1
        "#,
        params![name],
        |row| {
            let action_type_str: String = row.get(2)?;
            let action_type = ActionType::parse(&action_type_str).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    2,
                    "action_type".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            Ok(Action {
                name: row.get(0)?,
                action_point_cost: row.get(1)?,
                action_type,
                text: row.get(3)?,
            })
        },
    )
    .optional()
    .with_context(|| format!("Failed to fetch action `{}`", name))
}

fn validate_card(card: &Action) -> Result<()> {
    crate::shared::require_non_empty_trimmed("Action.name", &card.name)?;
    crate::shared::require_non_empty_trimmed("Action.text", &card.text)?;

    // Action point cost is a "number" in your spec; enforce non-negative by default.
    // If you later want to allow negative costs, remove this.
    if card.action_point_cost < 0 {
        anyhow::bail!("Action.action_point_cost must be >= 0");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "#,
        )
        .expect("create actions schema");
    }

    #[test]
    fn save_card_persists_to_database() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        let a = Action {
            name: "Strike".to_string(),
            action_point_cost: 2,
            action_type: ActionType::Attack,
            text: "Deal damage.".to_string(),
        };

        let saved = save_card(&conn, &a).expect("save_card should succeed");
        assert_eq!(saved, a);

        let reloaded = get_card(&conn, "Strike")
            .expect("get_card should succeed")
            .expect("saved card should exist");
        assert_eq!(reloaded, a);
    }

    #[test]
    fn get_card_returns_none_for_missing() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        let missing = get_card(&conn, "Missing").expect("get_card should succeed");
        assert!(missing.is_none());
    }

    #[test]
    fn update_card_persists_changes() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        let a1 = Action {
            name: "Strike".to_string(),
            action_point_cost: 2,
            action_type: ActionType::Attack,
            text: "Deal damage.".to_string(),
        };
        save_card(&conn, &a1).expect("save initial action");

        let a2 = Action {
            action_point_cost: 3,
            text: "Deal lots of damage.".to_string(),
            ..a1.clone()
        };

        let updated = update_card(&conn, &a2)
            .expect("update_card should succeed")
            .expect("row should exist to update");
        assert_eq!(updated, a2);

        let reloaded = get_card(&conn, "Strike")
            .expect("get_card should succeed")
            .expect("card should still exist");
        assert_eq!(reloaded, a2);
    }

    #[test]
    fn rename_and_update_card_renames_primary_key_and_updates_fields() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        let a1 = Action {
            name: "Strike".to_string(),
            action_point_cost: 2,
            action_type: ActionType::Attack,
            text: "Deal damage.".to_string(),
        };
        save_card(&conn, &a1).expect("save initial action");

        let a2 = Action {
            name: "Heavy Strike".to_string(),
            action_point_cost: 4,
            action_type: ActionType::Attack,
            text: "Deal heavy damage.".to_string(),
        };

        let renamed = rename_and_update_card(&conn, "Strike", &a2)
            .expect("rename_and_update_card should succeed")
            .expect("row should exist to rename");
        assert_eq!(renamed, a2);

        let old = get_card(&conn, "Strike").expect("get old after rename");
        assert!(old.is_none());

        let new = get_card(&conn, "Heavy Strike")
            .expect("get new after rename")
            .expect("renamed card should exist");
        assert_eq!(new, a2);
    }

    #[test]
    fn update_card_returns_none_if_missing() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        let a = Action {
            name: "Nope".to_string(),
            action_point_cost: 1,
            action_type: ActionType::Interaction,
            text: "Nothing.".to_string(),
        };

        let updated = update_card(&conn, &a).expect("update_card should succeed");
        assert!(updated.is_none());
    }

    #[test]
    fn delete_card_removes_row() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        let a = Action {
            name: "Strike".to_string(),
            action_point_cost: 2,
            action_type: ActionType::Attack,
            text: "Deal damage.".to_string(),
        };
        save_card(&conn, &a).expect("save action");

        let deleted = delete_card(&conn, "Strike").expect("delete_card should succeed");
        assert!(deleted);

        let after = get_card(&conn, "Strike").expect("get_card after delete");
        assert!(after.is_none());
    }

    #[test]
    fn delete_card_returns_false_if_missing() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        let deleted = delete_card(&conn, "Missing").expect("delete_card should succeed");
        assert!(!deleted);
    }
}
