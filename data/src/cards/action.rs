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

    fn to_db(self) -> &'static str {
        self.as_str()
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

/// Optional association for an Action.
///
/// At most one association should be present at a time. The DB schema enforces
/// this (see migration) and these APIs assume the same invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionAssociation {
    Unit { unit_name: String },
    Item { item_name: String },
    Level { level_name: String },
}

fn get_unit_id(conn: &Connection, unit_name: &str) -> Result<i64> {
    let unit_name = crate::shared::require_non_empty_trimmed("unit_name", unit_name)?;
    conn.query_row(
        "SELECT id FROM units WHERE name = ?1",
        params![unit_name],
        |row| row.get(0),
    )
    .with_context(|| format!("Failed to resolve Unit `{unit_name}` id"))
}

fn get_item_id(conn: &Connection, item_name: &str) -> Result<i64> {
    let item_name = crate::shared::require_non_empty_trimmed("item_name", item_name)?;
    conn.query_row(
        "SELECT id FROM items WHERE name = ?1",
        params![item_name],
        |row| row.get(0),
    )
    .with_context(|| format!("Failed to resolve Item `{item_name}` id"))
}

fn get_level_id(conn: &Connection, level_name: &str) -> Result<i64> {
    let level_name = crate::shared::require_non_empty_trimmed("level_name", level_name)?;
    conn.query_row(
        "SELECT id FROM levels WHERE name = ?1",
        params![level_name],
        |row| row.get(0),
    )
    .with_context(|| format!("Failed to resolve Level `{level_name}` id"))
}

/// Clears any Unit/Item/Level association for the given action.
pub fn clear_association(conn: &Connection, action_name: &str) -> Result<()> {
    let action_name = crate::shared::require_non_empty_trimmed("action_name", action_name)?;

    let changed = conn
        .execute(
            r#"
            UPDATE actions
            SET unit_id = NULL,
                item_id = NULL,
                level_id = NULL
            WHERE name = ?1
            "#,
            params![action_name],
        )
        .with_context(|| format!("Failed to clear association for action `{action_name}`"))?;

    if changed == 0 {
        anyhow::bail!("Action `{action_name}` not found");
    }

    Ok(())
}

/// Sets the action association to exactly one of Unit/Item/Level (and clears the others).
pub fn set_association(
    conn: &Connection,
    action_name: &str,
    association: &ActionAssociation,
) -> Result<()> {
    let action_name = crate::shared::require_non_empty_trimmed("action_name", action_name)?;

    match association {
        ActionAssociation::Unit { unit_name } => {
            let unit_id = get_unit_id(conn, unit_name)?;
            let changed = conn
                .execute(
                    r#"
                    UPDATE actions
                    SET unit_id = ?2,
                        item_id = NULL,
                        level_id = NULL
                    WHERE name = ?1
                    "#,
                    params![action_name, unit_id],
                )
                .with_context(|| {
                    format!(
                        "Failed to link action `{action_name}` to unit `{}`",
                        unit_name.trim()
                    )
                })?;
            if changed == 0 {
                anyhow::bail!("Action `{action_name}` not found");
            }
            Ok(())
        }
        ActionAssociation::Item { item_name } => {
            let item_id = get_item_id(conn, item_name)?;
            let changed = conn
                .execute(
                    r#"
                    UPDATE actions
                    SET item_id = ?2,
                        unit_id = NULL,
                        level_id = NULL
                    WHERE name = ?1
                    "#,
                    params![action_name, item_id],
                )
                .with_context(|| {
                    format!(
                        "Failed to link action `{action_name}` to item `{}`",
                        item_name.trim()
                    )
                })?;
            if changed == 0 {
                anyhow::bail!("Action `{action_name}` not found");
            }
            Ok(())
        }
        ActionAssociation::Level { level_name } => {
            let level_id = get_level_id(conn, level_name)?;
            let changed = conn
                .execute(
                    r#"
                    UPDATE actions
                    SET level_id = ?2,
                        unit_id = NULL,
                        item_id = NULL
                    WHERE name = ?1
                    "#,
                    params![action_name, level_id],
                )
                .with_context(|| {
                    format!(
                        "Failed to link action `{action_name}` to level `{}`",
                        level_name.trim()
                    )
                })?;
            if changed == 0 {
                anyhow::bail!("Action `{action_name}` not found");
            }
            Ok(())
        }
    }
}

/// Returns the association (if any) for a given action.
///
/// If the DB has no association columns (older schema), this will error when queried.
pub fn get_association(conn: &Connection, action_name: &str) -> Result<Option<ActionAssociation>> {
    let action_name = action_name.trim();
    if action_name.is_empty() {
        return Ok(None);
    }

    let assoc: Option<ActionAssociation> = conn
        .query_row(
            r#"
            SELECT
                u.name,
                i.name,
                l.name
            FROM actions a
            LEFT JOIN units  u ON u.id = a.unit_id
            LEFT JOIN items  i ON i.id = a.item_id
            LEFT JOIN levels l ON l.id = a.level_id
            WHERE a.name = ?1
            "#,
            params![action_name],
            |row| {
                let unit_name: Option<String> = row.get(0)?;
                let item_name: Option<String> = row.get(1)?;
                let level_name: Option<String> = row.get(2)?;

                let assoc = match (unit_name, item_name, level_name) {
                    (Some(unit_name), None, None) => Some(ActionAssociation::Unit { unit_name }),
                    (None, Some(item_name), None) => Some(ActionAssociation::Item { item_name }),
                    (None, None, Some(level_name)) => Some(ActionAssociation::Level { level_name }),
                    (None, None, None) => None,
                    _ => {
                        // This should be impossible if DB constraints/triggers are in place.
                        // Convert to a rusqlite-compatible error type for the row-mapper closure.
                        return Err(rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!(
                                    "Invalid action association state for `{}` (multiple links set)",
                                    action_name
                                ),
                            )),
                        ));
                    }
                };

                Ok(assoc)
            },
        )
        .optional()
        .with_context(|| format!("Failed to fetch association for action `{}`", action_name))?
        .flatten();

    Ok(assoc)
}

/// Lists all actions associated with a given Unit (by unit name), ordered by action name (ascending).
pub fn list_actions_for_unit(conn: &Connection, unit_name: &str) -> Result<Vec<Action>> {
    let unit_id = get_unit_id(conn, unit_name)?;
    list_actions_by_fk(conn, "unit_id", unit_id)
        .with_context(|| format!("Failed to list actions for Unit `{}`", unit_name.trim()))
}

/// Lists all actions associated with a given Item (by item name), ordered by action name (ascending).
pub fn list_actions_for_item(conn: &Connection, item_name: &str) -> Result<Vec<Action>> {
    let item_id = get_item_id(conn, item_name)?;
    list_actions_by_fk(conn, "item_id", item_id)
        .with_context(|| format!("Failed to list actions for Item `{}`", item_name.trim()))
}

/// Lists all actions associated with a given Level (by level name), ordered by action name (ascending).
pub fn list_actions_for_level(conn: &Connection, level_name: &str) -> Result<Vec<Action>> {
    let level_id = get_level_id(conn, level_name)?;
    list_actions_by_fk(conn, "level_id", level_id)
        .with_context(|| format!("Failed to list actions for Level `{}`", level_name.trim()))
}

fn list_actions_by_fk(conn: &Connection, fk_col: &str, fk_id: i64) -> Result<Vec<Action>> {
    // We intentionally keep this internal helper simple and safe:
    // - `fk_col` must be one of the known columns; otherwise we bail.
    match fk_col {
        "unit_id" | "item_id" | "level_id" => {}
        other => anyhow::bail!(
            "Invalid foreign key column `{}` for listing associated actions",
            other
        ),
    }

    let sql = format!(
        r#"
        SELECT name, action_point_cost, action_type, text
        FROM actions
        WHERE {} = ?1
        ORDER BY name ASC
        "#,
        fk_col
    );

    let mut stmt = conn
        .prepare(&sql)
        .with_context(|| format!("Failed to prepare list actions by `{}` query", fk_col))?;

    let rows = stmt
        .query_map(params![fk_id], |row| {
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
        .with_context(|| format!("Failed to query actions by `{}`", fk_col))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.with_context(|| "Failed to read action row")?);
    }
    Ok(out)
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
            card.action_type.to_db(),
            card.text
        ],
    )
    .with_context(|| format!("Failed to save action `{}`", card.name))?;

    get_card(conn, &card.name)?
        .with_context(|| format!("Action `{}` was saved but could not be reloaded", card.name))
}

/// Inserts a new action with an optional association to a Unit/Item/Level.
///
/// This is a convenience wrapper around `save_card` + `set_association`.
pub fn save_card_with_association(
    conn: &Connection,
    card: &Action,
    association: Option<&ActionAssociation>,
) -> Result<Action> {
    let saved = save_card(conn, card)?;
    if let Some(assoc) = association {
        set_association(conn, &saved.name, assoc)?;
    }

    match get_card(conn, &saved.name)? {
        Some(a) => Ok(a),
        None => anyhow::bail!("Action `{}` disappeared after save", saved.name),
    }
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
                card.action_type.to_db(),
                card.text
            ],
        )
        .with_context(|| format!("Failed to update action `{}`", card.name))?;

    if changed == 0 {
        return Ok(None);
    }

    get_card(conn, &card.name)
}

/// Updates an existing action and sets/clears its association.
pub fn update_card_with_association(
    conn: &Connection,
    card: &Action,
    association: Option<&ActionAssociation>,
) -> Result<Option<Action>> {
    let updated = update_card(conn, card)?;
    if updated.is_none() {
        return Ok(None);
    }

    match association {
        Some(assoc) => set_association(conn, &card.name, assoc)?,
        None => clear_association(conn, &card.name)?,
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
                card.action_type.to_db(),
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

                -- Optional association
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
            -- Enforce "at most one association" and validate referenced rows exist.
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
