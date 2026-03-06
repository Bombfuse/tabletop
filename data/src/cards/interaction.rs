use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skill {
    Strength,
    Focus,
    Intelligence,
    Knowledge,
    Agility,
}

impl Skill {
    fn as_str(self) -> &'static str {
        match self {
            Skill::Strength => "Strength",
            Skill::Focus => "Focus",
            Skill::Intelligence => "Intelligence",
            Skill::Knowledge => "Knowledge",
            Skill::Agility => "Agility",
        }
    }

    fn parse(s: &str) -> Result<Self> {
        match s {
            "Strength" => Ok(Skill::Strength),
            "Focus" => Ok(Skill::Focus),
            "Intelligence" => Ok(Skill::Intelligence),
            "Knowledge" => Ok(Skill::Knowledge),
            "Agility" => Ok(Skill::Agility),
            other => anyhow::bail!("Invalid Skill `{}`", other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interaction {
    /// The `actions.name` this interaction belongs to.
    pub action_name: String,

    pub range: i64,
    pub skill: Skill,
    /// 1-14 inclusive, or NULL.
    pub target: Option<i64>,
}

/// Lists interactions ordered by action name (ascending).
pub fn list_cards(conn: &Connection) -> Result<Vec<Interaction>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                a.name,
                i.range,
                i.skill,
                i.target
            FROM interactions i
            JOIN actions a ON a.id = i.action_id
            ORDER BY a.name ASC
            "#,
        )
        .with_context(|| "Failed to prepare list interactions query")?;

    let rows = stmt
        .query_map([], |row| {
            let skill_str: String = row.get(2)?;
            let skill = Skill::parse(&skill_str).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    2,
                    "skill".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            Ok(Interaction {
                action_name: row.get(0)?,
                range: row.get(1)?,
                skill,
                target: row.get(3)?,
            })
        })
        .with_context(|| "Failed to query interactions")?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.with_context(|| "Failed to read interaction row")?);
    }

    Ok(out)
}

/// Inserts a new interaction for an existing action (by action name).
///
/// - The action must exist.
/// - This assumes there is at most one interaction per action (enforced by `UNIQUE(action_id)`).
pub fn save_card(conn: &Connection, card: &Interaction) -> Result<Interaction> {
    validate_card(card)?;

    let action_id = get_action_id(conn, &card.action_name)?
        .with_context(|| format!("Action `{}` not found", card.action_name))?;

    conn.execute(
        r#"
        INSERT INTO interactions (action_id, range, skill, target)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        params![action_id, card.range, card.skill.as_str(), card.target],
    )
    .with_context(|| {
        format!(
            "Failed to save interaction for action `{}`",
            card.action_name
        )
    })?;

    get_card(conn, &card.action_name)?.with_context(|| {
        format!(
            "Interaction for action `{}` was saved but could not be reloaded",
            card.action_name
        )
    })
}

/// Updates an existing interaction (by action name).
///
/// Returns `Ok(None)` if:
/// - the action doesn't exist, or
/// - the action exists but has no interaction row yet.
pub fn update_card(conn: &Connection, card: &Interaction) -> Result<Option<Interaction>> {
    validate_card(card)?;

    let Some(action_id) = get_action_id(conn, &card.action_name)? else {
        return Ok(None);
    };

    let changed = conn
        .execute(
            r#"
            UPDATE interactions
            SET
                range = ?2,
                skill = ?3,
                target = ?4
            WHERE action_id = ?1
            "#,
            params![action_id, card.range, card.skill.as_str(), card.target],
        )
        .with_context(|| {
            format!(
                "Failed to update interaction for action `{}`",
                card.action_name
            )
        })?;

    if changed == 0 {
        return Ok(None);
    }

    get_card(conn, &card.action_name)
}

/// Renames an interaction's owning action (moves the association) and updates fields.
///
/// This is useful when the action is renamed elsewhere but you want to move/update the interaction row
/// to follow the rename.
///
/// - `old_action_name` identifies the existing action+interaction.
/// - `card.action_name` is the new action name.
/// - If `old_action_name == card.action_name`, behaves like `update_card`.
///
/// Returns `Ok(None)` if:
/// - `old_action_name` action doesn't exist, or
/// - it exists but has no interaction, or
/// - `card.action_name` action doesn't exist.
pub fn rename_and_update_card(
    conn: &Connection,
    old_action_name: &str,
    card: &Interaction,
) -> Result<Option<Interaction>> {
    let old_action_name =
        crate::shared::require_non_empty_trimmed("old_action_name", old_action_name)?;
    validate_card(card)?;

    if old_action_name == card.action_name.trim() {
        return update_card(conn, card);
    }

    let Some(old_action_id) = get_action_id(conn, old_action_name)? else {
        return Ok(None);
    };

    // Ensure the old interaction exists to move.
    let old_has_interaction: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM interactions WHERE action_id = ?1",
            params![old_action_id],
            |row| row.get(0),
        )
        .optional()
        .with_context(|| {
            format!(
                "Failed to check existing interaction for action `{}`",
                old_action_name
            )
        })?;
    if old_has_interaction.is_none() {
        return Ok(None);
    }

    let Some(new_action_id) = get_action_id(conn, &card.action_name)? else {
        return Ok(None);
    };

    let changed = conn
        .execute(
            r#"
            UPDATE interactions
            SET
                action_id = ?2,
                range = ?3,
                skill = ?4,
                target = ?5
            WHERE action_id = ?1
            "#,
            params![
                old_action_id,
                new_action_id,
                card.range,
                card.skill.as_str(),
                card.target
            ],
        )
        .with_context(|| {
            format!(
                "Failed to move/update interaction from action `{}` to `{}`",
                old_action_name, card.action_name
            )
        })?;

    if changed == 0 {
        return Ok(None);
    }

    get_card(conn, &card.action_name)
}

/// Deletes an interaction by action name.
///
/// Returns `Ok(true)` if a row was deleted, `Ok(false)` if nothing matched.
pub fn delete_card(conn: &Connection, action_name: &str) -> Result<bool> {
    let action_name = action_name.trim();
    if action_name.is_empty() {
        return Ok(false);
    }

    let Some(action_id) = get_action_id(conn, action_name)? else {
        return Ok(false);
    };

    let changed = conn
        .execute(
            "DELETE FROM interactions WHERE action_id = ?1",
            params![action_id],
        )
        .with_context(|| format!("Failed to delete interaction for action `{}`", action_name))?;
    Ok(changed > 0)
}

/// Loads an interaction by action name.
///
/// Returns `Ok(None)` if not found.
pub fn get_card(conn: &Connection, action_name: &str) -> Result<Option<Interaction>> {
    let action_name = action_name.trim();
    if action_name.is_empty() {
        return Ok(None);
    }

    conn.query_row(
        r#"
        SELECT
            a.name,
            i.range,
            i.skill,
            i.target
        FROM interactions i
        JOIN actions a ON a.id = i.action_id
        WHERE a.name = ?1
        "#,
        params![action_name],
        |row| {
            let skill_str: String = row.get(2)?;
            let skill = Skill::parse(&skill_str).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    2,
                    "skill".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            Ok(Interaction {
                action_name: row.get(0)?,
                range: row.get(1)?,
                skill,
                target: row.get(3)?,
            })
        },
    )
    .optional()
    .with_context(|| format!("Failed to fetch interaction for action `{}`", action_name))
}

fn get_action_id(conn: &Connection, action_name: &str) -> Result<Option<i64>> {
    let action_name = action_name.trim();
    if action_name.is_empty() {
        return Ok(None);
    }

    conn.query_row(
        "SELECT id FROM actions WHERE name = ?1",
        params![action_name],
        |row| row.get(0),
    )
    .optional()
    .with_context(|| format!("Failed to resolve action id for `{}`", action_name))
}

fn validate_card(card: &Interaction) -> Result<()> {
    crate::shared::require_non_empty_trimmed("Interaction.action_name", &card.action_name)?;

    if card.range < 0 {
        anyhow::bail!("Interaction.range must be >= 0");
    }

    if let Some(target) = card.target {
        if target < 1 || target > 14 {
            anyhow::bail!("Interaction.target must be between 1 and 14 (inclusive) when present");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::action::{Action, ActionType};

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

    fn seed_action(conn: &Connection, name: &str) {
        let a = Action {
            name: name.to_string(),
            action_point_cost: 1,
            action_type: ActionType::Interaction,
            text: "desc".to_string(),
        };
        crate::cards::action::save_card(conn, &a).expect("seed action");
    }

    #[test]
    fn save_card_persists_to_database() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        seed_action(&conn, "Talk");

        let i = Interaction {
            action_name: "Talk".to_string(),
            range: 2,
            skill: Skill::Knowledge,
            target: Some(9),
        };

        let saved = save_card(&conn, &i).expect("save_card should succeed");
        assert_eq!(saved, i);

        let reloaded = get_card(&conn, "Talk")
            .expect("get_card should succeed")
            .expect("saved card should exist");
        assert_eq!(reloaded, i);
    }

    #[test]
    fn save_card_allows_null_target() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        seed_action(&conn, "Look");

        let i = Interaction {
            action_name: "Look".to_string(),
            range: 5,
            skill: Skill::Focus,
            target: None,
        };

        let saved = save_card(&conn, &i).expect("save_card should succeed");
        assert_eq!(saved, i);

        let reloaded = get_card(&conn, "Look")
            .expect("get_card should succeed")
            .expect("saved card should exist");
        assert_eq!(reloaded, i);
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

        seed_action(&conn, "Talk");

        let i1 = Interaction {
            action_name: "Talk".to_string(),
            range: 2,
            skill: Skill::Knowledge,
            target: Some(9),
        };
        save_card(&conn, &i1).expect("save initial interaction");

        let i2 = Interaction {
            range: 3,
            skill: Skill::Intelligence,
            target: None,
            ..i1.clone()
        };

        let updated = update_card(&conn, &i2)
            .expect("update_card should succeed")
            .expect("row should exist to update");
        assert_eq!(updated, i2);

        let reloaded = get_card(&conn, "Talk")
            .expect("get_card should succeed")
            .expect("card should still exist");
        assert_eq!(reloaded, i2);
    }

    #[test]
    fn delete_card_removes_row() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        seed_action(&conn, "Talk");

        let i = Interaction {
            action_name: "Talk".to_string(),
            range: 2,
            skill: Skill::Knowledge,
            target: Some(9),
        };
        save_card(&conn, &i).expect("save interaction");

        let deleted = delete_card(&conn, "Talk").expect("delete_card should succeed");
        assert!(deleted);

        let after = get_card(&conn, "Talk").expect("get_card after delete");
        assert!(after.is_none());
    }

    #[test]
    fn delete_card_returns_false_if_action_missing() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        let deleted = delete_card(&conn, "Missing").expect("delete_card should succeed");
        assert!(!deleted);
    }
}
