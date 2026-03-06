use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

impl Suit {
    pub fn as_str(self) -> &'static str {
        match self {
            Suit::Spades => "Spades",
            Suit::Clubs => "Clubs",
            Suit::Diamonds => "Diamonds",
            Suit::Hearts => "Hearts",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "Spades" => Ok(Suit::Spades),
            "Clubs" => Ok(Suit::Clubs),
            "Diamonds" => Ok(Suit::Diamonds),
            "Hearts" => Ok(Suit::Hearts),
            other => bail!("Invalid suit: {other}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Arcane,
    Physical,
}

impl DamageType {
    pub fn as_str(self) -> &'static str {
        match self {
            DamageType::Arcane => "Arcane",
            DamageType::Physical => "Physical",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "Arcane" => Ok(DamageType::Arcane),
            "Physical" => Ok(DamageType::Physical),
            other => bail!("Invalid damage type: {other}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmorModifier {
    pub card_id: i64,
    pub value: i64,
    pub suit: Suit,
    pub damage_type: DamageType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmorModifierRow {
    pub id: i64,
    pub card_id: i64,
    pub value: i64,
    pub suit: Suit,
    pub damage_type: DamageType,

    // Association (at most one should be Some at a time, or both None).
    pub item_name: Option<String>,
    pub level_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorModifierAssociationKind {
    None,
    Item,
    Level,
}

/// Inserts a new armor modifier.
pub fn insert(conn: &Connection, armor: &ArmorModifier) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO armor_modifiers (card_id, value, suit, damage_type)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        params![
            armor.card_id,
            armor.value,
            armor.suit.as_str(),
            armor.damage_type.as_str()
        ],
    )
    .with_context(|| format!("insert armor modifier (card_id={})", armor.card_id))?;

    Ok(())
}

/// Inserts a new armor modifier and returns its new `id`.
pub fn insert_returning_id(conn: &Connection, armor: &ArmorModifier) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO armor_modifiers (card_id, value, suit, damage_type)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        params![
            armor.card_id,
            armor.value,
            armor.suit.as_str(),
            armor.damage_type.as_str()
        ],
    )
    .with_context(|| format!("insert armor modifier (card_id={})", armor.card_id))?;

    Ok(conn.last_insert_rowid())
}

/// Updates an existing armor modifier by id.
pub fn update_by_id(conn: &Connection, id: i64, armor: &ArmorModifier) -> Result<()> {
    let changed = conn
        .execute(
            r#"
            UPDATE armor_modifiers
            SET card_id      = ?2,
                value        = ?3,
                suit         = ?4,
                damage_type  = ?5
            WHERE id = ?1
            "#,
            params![
                id,
                armor.card_id,
                armor.value,
                armor.suit.as_str(),
                armor.damage_type.as_str()
            ],
        )
        .with_context(|| format!("update armor modifier id={id}"))?;

    if changed == 0 {
        bail!("Armor modifier not found (id={id})");
    }

    Ok(())
}

/// Deletes an armor modifier by id.
pub fn delete_by_id(conn: &Connection, id: i64) -> Result<()> {
    let changed = conn
        .execute(
            r#"
            DELETE FROM armor_modifiers
            WHERE id = ?1
            "#,
            params![id],
        )
        .with_context(|| format!("delete armor modifier id={id}"))?;

    if changed == 0 {
        bail!("Armor modifier not found (id={id})");
    }

    Ok(())
}

/// Gets an armor modifier by id, including any item/level association names.
pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<ArmorModifierRow>> {
    let row = conn
        .query_row(
            r#"
            SELECT
                am.id,
                am.card_id,
                am.value,
                am.suit,
                am.damage_type,
                i.name  AS item_name,
                l.name  AS level_name
            FROM armor_modifiers am
            LEFT JOIN item_armor_modifiers iam
                ON iam.armor_modifier_id = am.id
            LEFT JOIN items i
                ON i.id = iam.item_id
            LEFT JOIN level_armor_modifiers lam
                ON lam.armor_modifier_id = am.id
            LEFT JOIN levels l
                ON l.id = lam.level_id
            WHERE am.id = ?1
            "#,
            params![id],
            |r| {
                let suit_str: String = r.get(3)?;
                let suit = Suit::parse(&suit_str).map_err(|e| {
                    // Convert to a rusqlite-compatible error type for the row-mapper closure.
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("{e:#}"),
                        )),
                    )
                })?;

                let damage_type_str: String = r.get(4)?;
                let damage_type = DamageType::parse(&damage_type_str).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("{e:#}"),
                        )),
                    )
                })?;

                Ok(ArmorModifierRow {
                    id: r.get(0)?,
                    card_id: r.get(1)?,
                    value: r.get(2)?,
                    suit,
                    damage_type,
                    item_name: r.get(5)?,
                    level_name: r.get(6)?,
                })
            },
        )
        .optional()
        .with_context(|| format!("get armor modifier id={id}"))?;

    Ok(row)
}

/// Lists all armor modifiers (ordered by id), including any item/level association names.
pub fn list_all(conn: &Connection) -> Result<Vec<ArmorModifierRow>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                am.id,
                am.card_id,
                am.value,
                am.suit,
                am.damage_type,
                i.name  AS item_name,
                l.name  AS level_name
            FROM armor_modifiers am
            LEFT JOIN item_armor_modifiers iam
                ON iam.armor_modifier_id = am.id
            LEFT JOIN items i
                ON i.id = iam.item_id
            LEFT JOIN level_armor_modifiers lam
                ON lam.armor_modifier_id = am.id
            LEFT JOIN levels l
                ON l.id = lam.level_id
            ORDER BY am.id ASC
            "#,
        )
        .context("prepare list armor modifiers")?;

    let rows = stmt.query_map([], |r| {
        let suit_str: String = r.get(3)?;
        let suit = Suit::parse(&suit_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{e:#}"),
                )),
            )
        })?;

        let damage_type_str: String = r.get(4)?;
        let damage_type = DamageType::parse(&damage_type_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{e:#}"),
                )),
            )
        })?;

        Ok(ArmorModifierRow {
            id: r.get(0)?,
            card_id: r.get(1)?,
            value: r.get(2)?,
            suit,
            damage_type,
            item_name: r.get(5)?,
            level_name: r.get(6)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Lists armor modifiers for a specific item (by item name).
pub fn list_for_item(conn: &Connection, item_name: &str) -> Result<Vec<ArmorModifierRow>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                am.id,
                am.card_id,
                am.value,
                am.suit,
                am.damage_type,
                i.name  AS item_name,
                NULL    AS level_name
            FROM items i
            JOIN item_armor_modifiers iam
                ON iam.item_id = i.id
            JOIN armor_modifiers am
                ON am.id = iam.armor_modifier_id
            WHERE i.name = ?1
            ORDER BY am.id ASC
            "#,
        )
        .with_context(|| format!("prepare list armor modifiers for item `{item_name}`"))?;

    let rows = stmt.query_map(params![item_name], |r| {
        let suit_str: String = r.get(3)?;
        let suit = Suit::parse(&suit_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{e:#}"),
                )),
            )
        })?;

        let damage_type_str: String = r.get(4)?;
        let damage_type = DamageType::parse(&damage_type_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{e:#}"),
                )),
            )
        })?;

        Ok(ArmorModifierRow {
            id: r.get(0)?,
            card_id: r.get(1)?,
            value: r.get(2)?,
            suit,
            damage_type,
            item_name: r.get(5)?,
            level_name: r.get(6)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Lists armor modifiers for a specific level (by level name).
pub fn list_for_level(conn: &Connection, level_name: &str) -> Result<Vec<ArmorModifierRow>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                am.id,
                am.card_id,
                am.value,
                am.suit,
                am.damage_type,
                NULL   AS item_name,
                l.name AS level_name
            FROM levels l
            JOIN level_armor_modifiers lam
                ON lam.level_id = l.id
            JOIN armor_modifiers am
                ON am.id = lam.armor_modifier_id
            WHERE l.name = ?1
            ORDER BY am.id ASC
            "#,
        )
        .with_context(|| format!("prepare list armor modifiers for level `{level_name}`"))?;

    let rows = stmt.query_map(params![level_name], |r| {
        let suit_str: String = r.get(3)?;
        let suit = Suit::parse(&suit_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{e:#}"),
                )),
            )
        })?;

        let damage_type_str: String = r.get(4)?;
        let damage_type = DamageType::parse(&damage_type_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{e:#}"),
                )),
            )
        })?;

        Ok(ArmorModifierRow {
            id: r.get(0)?,
            card_id: r.get(1)?,
            value: r.get(2)?,
            suit,
            damage_type,
            item_name: r.get(5)?,
            level_name: r.get(6)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Returns which kind of association this armor modifier currently has.
pub fn get_association_kind(
    conn: &Connection,
    armor_modifier_id: i64,
) -> Result<ArmorModifierAssociationKind> {
    // We intentionally query both link tables. Triggers enforce exclusivity,
    // but we still handle "broken" states defensively.
    let item_link: Option<i64> = conn
        .query_row(
            r#"
            SELECT item_id
            FROM item_armor_modifiers
            WHERE armor_modifier_id = ?1
            "#,
            params![armor_modifier_id],
            |r| r.get(0),
        )
        .optional()
        .with_context(|| format!("get item link for armor_modifier_id={armor_modifier_id}"))?;

    let level_link: Option<i64> = conn
        .query_row(
            r#"
            SELECT level_id
            FROM level_armor_modifiers
            WHERE armor_modifier_id = ?1
            "#,
            params![armor_modifier_id],
            |r| r.get(0),
        )
        .optional()
        .with_context(|| format!("get level link for armor_modifier_id={armor_modifier_id}"))?;

    match (item_link, level_link) {
        (None, None) => Ok(ArmorModifierAssociationKind::None),
        (Some(_), None) => Ok(ArmorModifierAssociationKind::Item),
        (None, Some(_)) => Ok(ArmorModifierAssociationKind::Level),
        (Some(_), Some(_)) => {
            // Should be prevented by DB triggers.
            bail!("Invalid armor modifier association state (linked to both item and level)")
        }
    }
}

/// Links an armor modifier to an item by item name.
///
/// This will fail if:
/// - item name doesn't exist
/// - armor modifier doesn't exist
/// - armor modifier is already linked to a level (enforced by triggers)
/// - armor modifier is already linked to an item (UNIQUE on armor_modifier_id)
pub fn link_to_item_by_name(
    conn: &Connection,
    armor_modifier_id: i64,
    item_name: &str,
) -> Result<()> {
    let item_id: Option<i64> = conn
        .query_row(
            r#"
            SELECT id
            FROM items
            WHERE name = ?1
            "#,
            params![item_name],
            |r| r.get(0),
        )
        .optional()
        .with_context(|| format!("lookup item id for `{item_name}`"))?;

    let Some(item_id) = item_id else {
        bail!("Item not found: `{item_name}`");
    };

    // Ensure modifier exists (gives nicer error than FK failure).
    let exists: Option<i64> = conn
        .query_row(
            r#"
            SELECT id
            FROM armor_modifiers
            WHERE id = ?1
            "#,
            params![armor_modifier_id],
            |r| r.get(0),
        )
        .optional()
        .with_context(|| format!("check armor modifier exists id={armor_modifier_id}"))?;

    if exists.is_none() {
        bail!("Armor modifier not found (id={armor_modifier_id})");
    }

    conn.execute(
        r#"
        INSERT INTO item_armor_modifiers (item_id, armor_modifier_id)
        VALUES (?1, ?2)
        "#,
        params![item_id, armor_modifier_id],
    )
    .with_context(|| format!("link armor_modifier_id={armor_modifier_id} to item `{item_name}`"))?;

    Ok(())
}

/// Links an armor modifier to a level by level name.
///
/// This will fail if:
/// - level name doesn't exist
/// - armor modifier doesn't exist
/// - armor modifier is already linked to an item (enforced by triggers)
/// - armor modifier is already linked to a level (UNIQUE on armor_modifier_id)
pub fn link_to_level_by_name(
    conn: &Connection,
    armor_modifier_id: i64,
    level_name: &str,
) -> Result<()> {
    let level_id: Option<i64> = conn
        .query_row(
            r#"
            SELECT id
            FROM levels
            WHERE name = ?1
            "#,
            params![level_name],
            |r| r.get(0),
        )
        .optional()
        .with_context(|| format!("lookup level id for `{level_name}`"))?;

    let Some(level_id) = level_id else {
        bail!("Level not found: `{level_name}`");
    };

    // Ensure modifier exists (gives nicer error than FK failure).
    let exists: Option<i64> = conn
        .query_row(
            r#"
            SELECT id
            FROM armor_modifiers
            WHERE id = ?1
            "#,
            params![armor_modifier_id],
            |r| r.get(0),
        )
        .optional()
        .with_context(|| format!("check armor modifier exists id={armor_modifier_id}"))?;

    if exists.is_none() {
        bail!("Armor modifier not found (id={armor_modifier_id})");
    }

    conn.execute(
        r#"
        INSERT INTO level_armor_modifiers (level_id, armor_modifier_id)
        VALUES (?1, ?2)
        "#,
        params![level_id, armor_modifier_id],
    )
    .with_context(|| {
        format!("link armor_modifier_id={armor_modifier_id} to level `{level_name}`")
    })?;

    Ok(())
}

/// Removes any association (item or level) for the armor modifier id.
///
/// This is useful prior to re-linking.
pub fn clear_association(conn: &Connection, armor_modifier_id: i64) -> Result<()> {
    conn.execute(
        r#"
        DELETE FROM item_armor_modifiers
        WHERE armor_modifier_id = ?1
        "#,
        params![armor_modifier_id],
    )
    .with_context(|| format!("clear item link armor_modifier_id={armor_modifier_id}"))?;

    conn.execute(
        r#"
        DELETE FROM level_armor_modifiers
        WHERE armor_modifier_id = ?1
        "#,
        params![armor_modifier_id],
    )
    .with_context(|| format!("clear level link armor_modifier_id={armor_modifier_id}"))?;

    Ok(())
}

/// Convenience: create an armor modifier and link it to an item by name in a single transaction.
pub fn create_and_link_to_item_by_name(
    conn: &mut Connection,
    armor: &ArmorModifier,
    item_name: &str,
) -> Result<i64> {
    let tx = conn
        .transaction()
        .context("begin tx: create_and_link_to_item")?;

    let id = insert_returning_id(&tx, armor).context("insert armor modifier")?;
    link_to_item_by_name(&tx, id, item_name).context("link to item")?;

    tx.commit().context("commit tx: create_and_link_to_item")?;
    Ok(id)
}

/// Convenience: create an armor modifier and link it to a level by name in a single transaction.
pub fn create_and_link_to_level_by_name(
    conn: &mut Connection,
    armor: &ArmorModifier,
    level_name: &str,
) -> Result<i64> {
    let tx = conn
        .transaction()
        .context("begin tx: create_and_link_to_level")?;

    let id = insert_returning_id(&tx, armor).context("insert armor modifier")?;
    link_to_level_by_name(&tx, id, level_name).context("link to level")?;

    tx.commit().context("commit tx: create_and_link_to_level")?;
    Ok(id)
}
