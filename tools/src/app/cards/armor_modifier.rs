use anyhow::Result;
use rusqlite::Connection;

pub use data::cards::armor_modifier::{
    ArmorModifier, ArmorModifierAssociationKind, ArmorModifierRow, DamageType, Suit,
};

/// Lists armor modifiers ordered by id (ascending).
///
/// Delegates to the shared `data` crate.
pub fn list_all(conn: &Connection) -> Result<Vec<ArmorModifierRow>> {
    data::cards::armor_modifier::list_all(conn)
}

/// Gets an armor modifier by id (including any association names).
///
/// Delegates to the shared `data` crate.
pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<ArmorModifierRow>> {
    data::cards::armor_modifier::get_by_id(conn, id)
}

/// Inserts a new armor modifier.
///
/// Delegates to the shared `data` crate.
pub fn insert(conn: &Connection, armor: &ArmorModifier) -> Result<()> {
    data::cards::armor_modifier::insert(conn, armor)
}

/// Inserts a new armor modifier and returns its id.
///
/// Delegates to the shared `data` crate.
pub fn insert_returning_id(conn: &Connection, armor: &ArmorModifier) -> Result<i64> {
    data::cards::armor_modifier::insert_returning_id(conn, armor)
}

/// Updates an armor modifier by id.
///
/// Delegates to the shared `data` crate.
pub fn update_by_id(conn: &Connection, id: i64, armor: &ArmorModifier) -> Result<()> {
    data::cards::armor_modifier::update_by_id(conn, id, armor)
}

/// Deletes an armor modifier by id.
///
/// Delegates to the shared `data` crate.
pub fn delete_by_id(conn: &Connection, id: i64) -> Result<()> {
    data::cards::armor_modifier::delete_by_id(conn, id)
}

/// Lists armor modifiers associated with an item (by item name).
///
/// Delegates to the shared `data` crate.
pub fn list_for_item(conn: &Connection, item_name: &str) -> Result<Vec<ArmorModifierRow>> {
    data::cards::armor_modifier::list_for_item(conn, item_name)
}

/// Lists armor modifiers associated with a level (by level name).
///
/// Delegates to the shared `data` crate.
pub fn list_for_level(conn: &Connection, level_name: &str) -> Result<Vec<ArmorModifierRow>> {
    data::cards::armor_modifier::list_for_level(conn, level_name)
}

/// Returns the current association kind for an armor modifier id.
///
/// Delegates to the shared `data` crate.
pub fn get_association_kind(
    conn: &Connection,
    armor_modifier_id: i64,
) -> Result<ArmorModifierAssociationKind> {
    data::cards::armor_modifier::get_association_kind(conn, armor_modifier_id)
}

/// Links an armor modifier to an item by item name.
///
/// Delegates to the shared `data` crate.
pub fn link_to_item_by_name(
    conn: &Connection,
    armor_modifier_id: i64,
    item_name: &str,
) -> Result<()> {
    data::cards::armor_modifier::link_to_item_by_name(conn, armor_modifier_id, item_name)
}

/// Links an armor modifier to a level by level name.
///
/// Delegates to the shared `data` crate.
pub fn link_to_level_by_name(
    conn: &Connection,
    armor_modifier_id: i64,
    level_name: &str,
) -> Result<()> {
    data::cards::armor_modifier::link_to_level_by_name(conn, armor_modifier_id, level_name)
}

/// Clears any association (item or level) for an armor modifier id.
///
/// Delegates to the shared `data` crate.
pub fn clear_association(conn: &Connection, armor_modifier_id: i64) -> Result<()> {
    data::cards::armor_modifier::clear_association(conn, armor_modifier_id)
}

/// Convenience: create an armor modifier and link it to an item by name in one transaction.
///
/// Delegates to the shared `data` crate.
pub fn create_and_link_to_item_by_name(
    conn: &mut Connection,
    armor: &ArmorModifier,
    item_name: &str,
) -> Result<i64> {
    data::cards::armor_modifier::create_and_link_to_item_by_name(conn, armor, item_name)
}

/// Convenience: create an armor modifier and link it to a level by name in one transaction.
///
/// Delegates to the shared `data` crate.
pub fn create_and_link_to_level_by_name(
    conn: &mut Connection,
    armor: &ArmorModifier,
    level_name: &str,
) -> Result<i64> {
    data::cards::armor_modifier::create_and_link_to_level_by_name(conn, armor, level_name)
}
