use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Arcane,
    Physical,
}

impl DamageType {
    fn as_str(self) -> &'static str {
        match self {
            DamageType::Arcane => "Arcane",
            DamageType::Physical => "Physical",
        }
    }

    fn parse(s: &str) -> Result<Self> {
        match s {
            "Arcane" => Ok(DamageType::Arcane),
            "Physical" => Ok(DamageType::Physical),
            other => anyhow::bail!("Invalid DamageType `{}`", other),
        }
    }
}

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
pub struct Attack {
    /// The `actions.name` this attack belongs to.
    pub action_name: String,

    pub damage: i64,
    pub damage_type: DamageType,
    pub skill: Skill,
    /// 1-14 inclusive.
    pub target: i64,
    pub range: i64,
}

/// Lists attacks ordered by action name (ascending).
pub fn list_cards(conn: &Connection) -> Result<Vec<Attack>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                a.name,
                atk.damage,
                atk.damage_type,
                atk.skill,
                atk.target,
                atk.range
            FROM attacks atk
            JOIN actions a ON a.id = atk.action_id
            ORDER BY a.name ASC
            "#,
        )
        .with_context(|| "Failed to prepare list attacks query")?;

    let rows = stmt
        .query_map([], |row| {
            let damage_type_str: String = row.get(2)?;
            let skill_str: String = row.get(3)?;

            let damage_type = DamageType::parse(&damage_type_str).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    2,
                    "damage_type".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let skill = Skill::parse(&skill_str).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    3,
                    "skill".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;

            Ok(Attack {
                action_name: row.get(0)?,
                damage: row.get(1)?,
                damage_type,
                skill,
                target: row.get(4)?,
                range: row.get(5)?,
            })
        })
        .with_context(|| "Failed to query attacks")?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.with_context(|| "Failed to read attack row")?);
    }

    Ok(out)
}

/// Inserts a new attack for an existing action (by action name).
///
/// - The action must exist.
/// - This assumes there is at most one attack per action (enforced by `UNIQUE(action_id)`).
pub fn save_card(conn: &Connection, card: &Attack) -> Result<Attack> {
    validate_card(card)?;

    let action_id = get_action_id(conn, &card.action_name)?
        .with_context(|| format!("Action `{}` not found", card.action_name))?;

    conn.execute(
        r#"
        INSERT INTO attacks (action_id, damage, damage_type, skill, target, range)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![
            action_id,
            card.damage,
            card.damage_type.as_str(),
            card.skill.as_str(),
            card.target,
            card.range
        ],
    )
    .with_context(|| format!("Failed to save attack for action `{}`", card.action_name))?;

    get_card(conn, &card.action_name)?.with_context(|| {
        format!(
            "Attack for action `{}` was saved but could not be reloaded",
            card.action_name
        )
    })
}

/// Updates an existing attack (by action name).
///
/// Returns `Ok(None)` if:
/// - the action doesn't exist, or
/// - the action exists but has no attack row yet.
pub fn update_card(conn: &Connection, card: &Attack) -> Result<Option<Attack>> {
    validate_card(card)?;

    let Some(action_id) = get_action_id(conn, &card.action_name)? else {
        return Ok(None);
    };

    let changed = conn
        .execute(
            r#"
            UPDATE attacks
            SET
                damage = ?2,
                damage_type = ?3,
                skill = ?4,
                target = ?5,
                range = ?6
            WHERE action_id = ?1
            "#,
            params![
                action_id,
                card.damage,
                card.damage_type.as_str(),
                card.skill.as_str(),
                card.target,
                card.range
            ],
        )
        .with_context(|| format!("Failed to update attack for action `{}`", card.action_name))?;

    if changed == 0 {
        return Ok(None);
    }

    get_card(conn, &card.action_name)
}

/// Renames an attack's owning action (moves the association) and updates fields.
///
/// This is useful when the action is renamed elsewhere but you want to move/update the attack row
/// to follow the rename.
///
/// - `old_action_name` identifies the existing action+attack.
/// - `card.action_name` is the new action name.
/// - If `old_action_name == card.action_name`, behaves like `update_card`.
///
/// Returns `Ok(None)` if:
/// - `old_action_name` action doesn't exist, or
/// - it exists but has no attack, or
/// - `card.action_name` action doesn't exist.
pub fn rename_and_update_card(
    conn: &Connection,
    old_action_name: &str,
    card: &Attack,
) -> Result<Option<Attack>> {
    let old_action_name =
        crate::shared::require_non_empty_trimmed("old_action_name", old_action_name)?;
    validate_card(card)?;

    if old_action_name == card.action_name.trim() {
        return update_card(conn, card);
    }

    let Some(old_action_id) = get_action_id(conn, old_action_name)? else {
        return Ok(None);
    };

    // Ensure the old attack exists to move.
    let old_has_attack: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM attacks WHERE action_id = ?1",
            params![old_action_id],
            |row| row.get(0),
        )
        .optional()
        .with_context(|| {
            format!(
                "Failed to check existing attack for action `{}`",
                old_action_name
            )
        })?;
    if old_has_attack.is_none() {
        return Ok(None);
    }

    let Some(new_action_id) = get_action_id(conn, &card.action_name)? else {
        return Ok(None);
    };

    let changed = conn
        .execute(
            r#"
            UPDATE attacks
            SET
                action_id = ?2,
                damage = ?3,
                damage_type = ?4,
                skill = ?5,
                target = ?6,
                range = ?7
            WHERE action_id = ?1
            "#,
            params![
                old_action_id,
                new_action_id,
                card.damage,
                card.damage_type.as_str(),
                card.skill.as_str(),
                card.target,
                card.range
            ],
        )
        .with_context(|| {
            format!(
                "Failed to move/update attack from action `{}` to `{}`",
                old_action_name, card.action_name
            )
        })?;

    if changed == 0 {
        return Ok(None);
    }

    get_card(conn, &card.action_name)
}

/// Deletes an attack by action name.
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
            "DELETE FROM attacks WHERE action_id = ?1",
            params![action_id],
        )
        .with_context(|| format!("Failed to delete attack for action `{}`", action_name))?;
    Ok(changed > 0)
}

/// Loads an attack by action name.
///
/// Returns `Ok(None)` if not found.
pub fn get_card(conn: &Connection, action_name: &str) -> Result<Option<Attack>> {
    let action_name = action_name.trim();
    if action_name.is_empty() {
        return Ok(None);
    }

    conn.query_row(
        r#"
        SELECT
            a.name,
            atk.damage,
            atk.damage_type,
            atk.skill,
            atk.target,
            atk.range
        FROM attacks atk
        JOIN actions a ON a.id = atk.action_id
        WHERE a.name = ?1
        "#,
        params![action_name],
        |row| {
            let damage_type_str: String = row.get(2)?;
            let skill_str: String = row.get(3)?;

            let damage_type = DamageType::parse(&damage_type_str).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    2,
                    "damage_type".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let skill = Skill::parse(&skill_str).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    3,
                    "skill".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;

            Ok(Attack {
                action_name: row.get(0)?,
                damage: row.get(1)?,
                damage_type,
                skill,
                target: row.get(4)?,
                range: row.get(5)?,
            })
        },
    )
    .optional()
    .with_context(|| format!("Failed to fetch attack for action `{}`", action_name))
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

fn validate_card(card: &Attack) -> Result<()> {
    crate::shared::require_non_empty_trimmed("Attack.action_name", &card.action_name)?;

    if card.target < 1 || card.target > 14 {
        anyhow::bail!("Attack.target must be between 1 and 14 (inclusive)");
    }

    // Damage/range are "number" in your spec; enforce sane non-negative defaults.
    if card.damage < 0 {
        anyhow::bail!("Attack.damage must be >= 0");
    }
    if card.range < 0 {
        anyhow::bail!("Attack.range must be >= 0");
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
            "#,
        )
        .expect("create schema");
    }

    fn seed_action(conn: &Connection, name: &str) {
        let a = Action {
            name: name.to_string(),
            action_point_cost: 1,
            action_type: ActionType::Attack,
            text: "desc".to_string(),
        };
        crate::cards::action::save_card(conn, &a).expect("seed action");
    }

    #[test]
    fn save_card_persists_to_database() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        seed_action(&conn, "Strike");

        let atk = Attack {
            action_name: "Strike".to_string(),
            damage: 3,
            damage_type: DamageType::Physical,
            skill: Skill::Strength,
            target: 10,
            range: 1,
        };

        let saved = save_card(&conn, &atk).expect("save_card should succeed");
        assert_eq!(saved, atk);

        let reloaded = get_card(&conn, "Strike")
            .expect("get_card should succeed")
            .expect("saved card should exist");
        assert_eq!(reloaded, atk);
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

        seed_action(&conn, "Strike");

        let atk1 = Attack {
            action_name: "Strike".to_string(),
            damage: 3,
            damage_type: DamageType::Physical,
            skill: Skill::Strength,
            target: 10,
            range: 1,
        };
        save_card(&conn, &atk1).expect("save initial attack");

        let atk2 = Attack {
            damage: 7,
            damage_type: DamageType::Arcane,
            skill: Skill::Focus,
            target: 12,
            range: 2,
            ..atk1.clone()
        };

        let updated = update_card(&conn, &atk2)
            .expect("update_card should succeed")
            .expect("row should exist to update");
        assert_eq!(updated, atk2);

        let reloaded = get_card(&conn, "Strike")
            .expect("get_card should succeed")
            .expect("card should still exist");
        assert_eq!(reloaded, atk2);
    }

    #[test]
    fn delete_card_removes_row() {
        let conn = crate::cards::test_support::open_in_memory_db();
        create_schema(&conn);

        seed_action(&conn, "Strike");

        let atk = Attack {
            action_name: "Strike".to_string(),
            damage: 3,
            damage_type: DamageType::Physical,
            skill: Skill::Strength,
            target: 10,
            range: 1,
        };
        save_card(&conn, &atk).expect("save attack");

        let deleted = delete_card(&conn, "Strike").expect("delete_card should succeed");
        assert!(deleted);

        let after = get_card(&conn, "Strike").expect("get_card after delete");
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
