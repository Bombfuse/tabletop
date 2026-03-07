use anyhow::{Context, Result, anyhow};
use rusqlite::{Connection, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Strength,
    Focus,
    Intelligence,
    Knowledge,
    Agility,
}

impl Stat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Stat::Strength => "Strength",
            Stat::Focus => "Focus",
            Stat::Intelligence => "Intelligence",
            Stat::Knowledge => "Knowledge",
            Stat::Agility => "Agility",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "Strength" => Ok(Stat::Strength),
            "Focus" => Ok(Stat::Focus),
            "Intelligence" => Ok(Stat::Intelligence),
            "Knowledge" => Ok(Stat::Knowledge),
            "Agility" => Ok(Stat::Agility),
            other => Err(anyhow!("invalid Stat: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatModifierOperator {
    Add,
    Subtract,
}

impl StatModifierOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatModifierOperator::Add => "Add",
            StatModifierOperator::Subtract => "Subtract",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "Add" => Ok(StatModifierOperator::Add),
            "Subtract" => Ok(StatModifierOperator::Subtract),
            other => Err(anyhow!("invalid StatModifierOperator: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatModifier {
    pub stat: Stat,
    pub value: i64,
    pub operator: StatModifierOperator,
}

/// Row returned from list/get APIs, includes optional association names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatModifierRow {
    pub id: i64,
    pub stat: String,
    pub value: i64,
    pub operator: String,

    // Optional association (at most one should be Some at a time, or all None).
    pub unit_name: Option<String>,
    pub item_name: Option<String>,
    pub level_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatModifierAssociationKind {
    None,
    Unit,
    Item,
    Level,
}

pub fn insert(conn: &Connection, sm: &StatModifier) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO stat_modifiers (stat, value, operator)
        VALUES (?1, ?2, ?3)
        "#,
        params![sm.stat.as_str(), sm.value, sm.operator.as_str()],
    )?;
    Ok(())
}

pub fn insert_returning_id(conn: &Connection, sm: &StatModifier) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO stat_modifiers (stat, value, operator)
        VALUES (?1, ?2, ?3)
        "#,
        params![sm.stat.as_str(), sm.value, sm.operator.as_str()],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn update_by_id(conn: &Connection, id: i64, sm: &StatModifier) -> Result<()> {
    let changed = conn.execute(
        r#"
        UPDATE stat_modifiers
        SET stat = ?1, value = ?2, operator = ?3
        WHERE id = ?4
        "#,
        params![sm.stat.as_str(), sm.value, sm.operator.as_str(), id],
    )?;

    if changed == 0 {
        return Err(anyhow!("no stat_modifiers row found for id={id}"));
    }

    Ok(())
}

pub fn delete_by_id(conn: &Connection, id: i64) -> Result<()> {
    // Link rows are ON DELETE CASCADE on stat_modifiers, so deleting the base row is enough.
    let changed = conn.execute(
        r#"
        DELETE FROM stat_modifiers
        WHERE id = ?1
        "#,
        params![id],
    )?;

    if changed == 0 {
        return Err(anyhow!("no stat_modifiers row found for id={id}"));
    }

    Ok(())
}

pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<StatModifierRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            sm.id,
            sm.stat,
            sm.value,
            sm.operator,

            u.name AS unit_name,
            i.name AS item_name,
            l.name AS level_name
        FROM stat_modifiers sm
        LEFT JOIN unit_stat_modifiers usm ON usm.stat_modifier_id = sm.id
        LEFT JOIN units u ON u.id = usm.unit_id
        LEFT JOIN item_stat_modifiers ism ON ism.stat_modifier_id = sm.id
        LEFT JOIN items i ON i.id = ism.item_id
        LEFT JOIN level_stat_modifiers lsm ON lsm.stat_modifier_id = sm.id
        LEFT JOIN levels l ON l.id = lsm.level_id
        WHERE sm.id = ?1
        "#,
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(StatModifierRow {
            id: row.get(0)?,
            stat: row.get(1)?,
            value: row.get(2)?,
            operator: row.get(3)?,
            unit_name: row.get(4)?,
            item_name: row.get(5)?,
            level_name: row.get(6)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_all(conn: &Connection) -> Result<Vec<StatModifierRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            sm.id,
            sm.stat,
            sm.value,
            sm.operator,

            u.name AS unit_name,
            i.name AS item_name,
            l.name AS level_name
        FROM stat_modifiers sm
        LEFT JOIN unit_stat_modifiers usm ON usm.stat_modifier_id = sm.id
        LEFT JOIN units u ON u.id = usm.unit_id
        LEFT JOIN item_stat_modifiers ism ON ism.stat_modifier_id = sm.id
        LEFT JOIN items i ON i.id = ism.item_id
        LEFT JOIN level_stat_modifiers lsm ON lsm.stat_modifier_id = sm.id
        LEFT JOIN levels l ON l.id = lsm.level_id
        ORDER BY sm.id ASC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(StatModifierRow {
            id: row.get(0)?,
            stat: row.get(1)?,
            value: row.get(2)?,
            operator: row.get(3)?,
            unit_name: row.get(4)?,
            item_name: row.get(5)?,
            level_name: row.get(6)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn list_for_unit(conn: &Connection, unit_name: &str) -> Result<Vec<StatModifierRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            sm.id,
            sm.stat,
            sm.value,
            sm.operator,

            u.name AS unit_name,
            NULL AS item_name,
            NULL AS level_name
        FROM unit_stat_modifiers usm
        JOIN units u ON u.id = usm.unit_id
        JOIN stat_modifiers sm ON sm.id = usm.stat_modifier_id
        WHERE u.name = ?1
        ORDER BY sm.id ASC
        "#,
    )?;

    let rows = stmt.query_map(params![unit_name], |row| {
        Ok(StatModifierRow {
            id: row.get(0)?,
            stat: row.get(1)?,
            value: row.get(2)?,
            operator: row.get(3)?,
            unit_name: row.get(4)?,
            item_name: row.get(5)?,
            level_name: row.get(6)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn list_for_item(conn: &Connection, item_name: &str) -> Result<Vec<StatModifierRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            sm.id,
            sm.stat,
            sm.value,
            sm.operator,

            NULL AS unit_name,
            i.name AS item_name,
            NULL AS level_name
        FROM item_stat_modifiers ism
        JOIN items i ON i.id = ism.item_id
        JOIN stat_modifiers sm ON sm.id = ism.stat_modifier_id
        WHERE i.name = ?1
        ORDER BY sm.id ASC
        "#,
    )?;

    let rows = stmt.query_map(params![item_name], |row| {
        Ok(StatModifierRow {
            id: row.get(0)?,
            stat: row.get(1)?,
            value: row.get(2)?,
            operator: row.get(3)?,
            unit_name: row.get(4)?,
            item_name: row.get(5)?,
            level_name: row.get(6)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn list_for_level(conn: &Connection, level_name: &str) -> Result<Vec<StatModifierRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            sm.id,
            sm.stat,
            sm.value,
            sm.operator,

            NULL AS unit_name,
            NULL AS item_name,
            l.name AS level_name
        FROM level_stat_modifiers lsm
        JOIN levels l ON l.id = lsm.level_id
        JOIN stat_modifiers sm ON sm.id = lsm.stat_modifier_id
        WHERE l.name = ?1
        ORDER BY sm.id ASC
        "#,
    )?;

    let rows = stmt.query_map(params![level_name], |row| {
        Ok(StatModifierRow {
            id: row.get(0)?,
            stat: row.get(1)?,
            value: row.get(2)?,
            operator: row.get(3)?,
            unit_name: row.get(4)?,
            item_name: row.get(5)?,
            level_name: row.get(6)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get_association_kind(
    conn: &Connection,
    stat_modifier_id: i64,
) -> Result<StatModifierAssociationKind> {
    let unit_link: bool = conn.query_row(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM unit_stat_modifiers WHERE stat_modifier_id = ?1
        )
        "#,
        params![stat_modifier_id],
        |row| row.get(0),
    )?;

    let item_link: bool = conn.query_row(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM item_stat_modifiers WHERE stat_modifier_id = ?1
        )
        "#,
        params![stat_modifier_id],
        |row| row.get(0),
    )?;

    let level_link: bool = conn.query_row(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM level_stat_modifiers WHERE stat_modifier_id = ?1
        )
        "#,
        params![stat_modifier_id],
        |row| row.get(0),
    )?;

    let count = (unit_link as i32) + (item_link as i32) + (level_link as i32);
    if count == 0 {
        return Ok(StatModifierAssociationKind::None);
    }
    if count > 1 {
        return Err(anyhow!(
            "stat_modifier id={stat_modifier_id} is linked to multiple card types (corrupt state)"
        ));
    }

    if unit_link {
        Ok(StatModifierAssociationKind::Unit)
    } else if item_link {
        Ok(StatModifierAssociationKind::Item)
    } else {
        Ok(StatModifierAssociationKind::Level)
    }
}

pub fn link_to_unit_by_name(
    conn: &Connection,
    stat_modifier_id: i64,
    unit_name: &str,
) -> Result<()> {
    // Resolve unit id
    let unit_id: i64 = conn
        .query_row(
            r#"
            SELECT id
            FROM units
            WHERE name = ?1
            "#,
            params![unit_name],
            |row| row.get(0),
        )
        .with_context(|| format!("resolve unit id by name: {unit_name}"))?;

    conn.execute(
        r#"
        INSERT INTO unit_stat_modifiers (unit_id, stat_modifier_id)
        VALUES (?1, ?2)
        "#,
        params![unit_id, stat_modifier_id],
    )?;

    Ok(())
}

pub fn link_to_item_by_name(
    conn: &Connection,
    stat_modifier_id: i64,
    item_name: &str,
) -> Result<()> {
    // Resolve item id
    let item_id: i64 = conn
        .query_row(
            r#"
            SELECT id
            FROM items
            WHERE name = ?1
            "#,
            params![item_name],
            |row| row.get(0),
        )
        .with_context(|| format!("resolve item id by name: {item_name}"))?;

    conn.execute(
        r#"
        INSERT INTO item_stat_modifiers (item_id, stat_modifier_id)
        VALUES (?1, ?2)
        "#,
        params![item_id, stat_modifier_id],
    )?;

    Ok(())
}

pub fn link_to_level_by_name(
    conn: &Connection,
    stat_modifier_id: i64,
    level_name: &str,
) -> Result<()> {
    // Resolve level id
    let level_id: i64 = conn
        .query_row(
            r#"
            SELECT id
            FROM levels
            WHERE name = ?1
            "#,
            params![level_name],
            |row| row.get(0),
        )
        .with_context(|| format!("resolve level id by name: {level_name}"))?;

    conn.execute(
        r#"
        INSERT INTO level_stat_modifiers (level_id, stat_modifier_id)
        VALUES (?1, ?2)
        "#,
        params![level_id, stat_modifier_id],
    )?;

    Ok(())
}

pub fn clear_association(conn: &Connection, stat_modifier_id: i64) -> Result<()> {
    conn.execute(
        r#"
        DELETE FROM unit_stat_modifiers
        WHERE stat_modifier_id = ?1
        "#,
        params![stat_modifier_id],
    )?;

    conn.execute(
        r#"
        DELETE FROM item_stat_modifiers
        WHERE stat_modifier_id = ?1
        "#,
        params![stat_modifier_id],
    )?;

    conn.execute(
        r#"
        DELETE FROM level_stat_modifiers
        WHERE stat_modifier_id = ?1
        "#,
        params![stat_modifier_id],
    )?;

    Ok(())
}

pub fn create_and_link_to_unit_by_name(
    conn: &mut Connection,
    sm: &StatModifier,
    unit_name: &str,
) -> Result<i64> {
    let tx = conn.transaction()?;
    let id = insert_returning_id(&tx, sm)?;
    link_to_unit_by_name(&tx, id, unit_name)?;
    tx.commit()?;
    Ok(id)
}

pub fn create_and_link_to_item_by_name(
    conn: &mut Connection,
    sm: &StatModifier,
    item_name: &str,
) -> Result<i64> {
    let tx = conn.transaction()?;
    let id = insert_returning_id(&tx, sm)?;
    link_to_item_by_name(&tx, id, item_name)?;
    tx.commit()?;
    Ok(id)
}

pub fn create_and_link_to_level_by_name(
    conn: &mut Connection,
    sm: &StatModifier,
    level_name: &str,
) -> Result<i64> {
    let tx = conn.transaction()?;
    let id = insert_returning_id(&tx, sm)?;
    link_to_level_by_name(&tx, id, level_name)?;
    tx.commit()?;
    Ok(id)
}
