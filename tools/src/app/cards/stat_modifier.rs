use anyhow::Result;
use rusqlite::Connection;

pub use data::cards::stat_modifier::{
    Stat, StatModifier, StatModifierAssociationKind, StatModifierOperator, StatModifierRow,
};

/// Lists stat modifiers ordered by id (ascending).
///
/// Delegates to the shared `data` crate.
pub fn list_all(conn: &Connection) -> Result<Vec<StatModifierRow>> {
    data::cards::stat_modifier::list_all(conn)
}

/// Gets a stat modifier by id (including any association names).
///
/// Delegates to the shared `data` crate.
pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<StatModifierRow>> {
    data::cards::stat_modifier::get_by_id(conn, id)
}

/// Inserts a new stat modifier.
///
/// Delegates to the shared `data` crate.
pub fn insert(conn: &Connection, stat_modifier: &StatModifier) -> Result<()> {
    data::cards::stat_modifier::insert(conn, stat_modifier)
}

/// Inserts a new stat modifier and returns its id.
///
/// Delegates to the shared `data` crate.
pub fn insert_returning_id(conn: &Connection, stat_modifier: &StatModifier) -> Result<i64> {
    data::cards::stat_modifier::insert_returning_id(conn, stat_modifier)
}

/// Updates a stat modifier by id.
///
/// Delegates to the shared `data` crate.
pub fn update_by_id(conn: &Connection, id: i64, stat_modifier: &StatModifier) -> Result<()> {
    data::cards::stat_modifier::update_by_id(conn, id, stat_modifier)
}

/// Deletes a stat modifier by id.
///
/// Delegates to the shared `data` crate.
pub fn delete_by_id(conn: &Connection, id: i64) -> Result<()> {
    data::cards::stat_modifier::delete_by_id(conn, id)
}

/// Lists stat modifiers associated with a unit (by unit name).
///
/// Delegates to the shared `data` crate.
pub fn list_for_unit(conn: &Connection, unit_name: &str) -> Result<Vec<StatModifierRow>> {
    data::cards::stat_modifier::list_for_unit(conn, unit_name)
}

/// Lists stat modifiers associated with an item (by item name).
///
/// Delegates to the shared `data` crate.
pub fn list_for_item(conn: &Connection, item_name: &str) -> Result<Vec<StatModifierRow>> {
    data::cards::stat_modifier::list_for_item(conn, item_name)
}

/// Lists stat modifiers associated with a level (by level name).
///
/// Delegates to the shared `data` crate.
pub fn list_for_level(conn: &Connection, level_name: &str) -> Result<Vec<StatModifierRow>> {
    data::cards::stat_modifier::list_for_level(conn, level_name)
}

/// Returns the current association kind for a stat modifier id.
///
/// Delegates to the shared `data` crate.
pub fn get_association_kind(
    conn: &Connection,
    stat_modifier_id: i64,
) -> Result<StatModifierAssociationKind> {
    data::cards::stat_modifier::get_association_kind(conn, stat_modifier_id)
}

/// Links a stat modifier to a unit by unit name.
///
/// Delegates to the shared `data` crate.
pub fn link_to_unit_by_name(conn: &Connection, stat_modifier_id: i64, unit_name: &str) -> Result<()> {
    data::cards::stat_modifier::link_to_unit_by_name(conn, stat_modifier_id, unit_name)
}

/// Links a stat modifier to an item by item name.
///
/// Delegates to the shared `data` crate.
pub fn link_to_item_by_name(conn: &Connection, stat_modifier_id: i64, item_name: &str) -> Result<()> {
    data::cards::stat_modifier::link_to_item_by_name(conn, stat_modifier_id, item_name)
}

/// Links a stat modifier to a level by level name.
///
/// Delegates to the shared `data` crate.
pub fn link_to_level_by_name(
    conn: &Connection,
    stat_modifier_id: i64,
    level_name: &str,
) -> Result<()> {
    data::cards::stat_modifier::link_to_level_by_name(conn, stat_modifier_id, level_name)
}

/// Clears any association (unit, item, or level) for a stat modifier id.
///
/// Delegates to the shared `data` crate.
pub fn clear_association(conn: &Connection, stat_modifier_id: i64) -> Result<()> {
    data::cards::stat_modifier::clear_association(conn, stat_modifier_id)
}

/// Convenience: create a stat modifier and link it to a unit by name in one transaction.
///
/// Delegates to the shared `data` crate.
pub fn create_and_link_to_unit_by_name(
    conn: &mut Connection,
    stat_modifier: &StatModifier,
    unit_name: &str,
) -> Result<i64> {
    data::cards::stat_modifier::create_and_link_to_unit_by_name(conn, stat_modifier, unit_name)
}

/// Convenience: create a stat modifier and link it to an item by name in one transaction.
///
/// Delegates to the shared `data` crate.
pub fn create_and_link_to_item_by_name(
    conn: &mut Connection,
    stat_modifier: &StatModifier,
    item_name: &str,
) -> Result<i64> {
    data::cards::stat_modifier::create_and_link_to_item_by_name(conn, stat_modifier, item_name)
}

/// Convenience: create a stat modifier and link it to a level by name in one transaction.
///
/// Delegates to the shared `data` crate.
pub fn create_and_link_to_level_by_name(
    conn: &mut Connection,
    stat_modifier: &StatModifier,
    level_name: &str,
) -> Result<i64> {
    data::cards::stat_modifier::create_and_link_to_level_by_name(conn, stat_modifier, level_name)
}
