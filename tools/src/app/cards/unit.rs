use anyhow::Result;
use rusqlite::Connection;

pub use data::cards::unit::Unit;

/// Lists units ordered by name (ascending).
///
/// Delegates to the shared `data` crate.
pub fn list_cards(conn: &Connection) -> Result<Vec<Unit>> {
    data::cards::unit::list_cards(conn)
}

/// Inserts a new unit.
///
/// Delegates to the shared `data` crate.
pub fn save_card(conn: &Connection, card: &Unit) -> Result<Unit> {
    data::cards::unit::save_card(conn, card)
}

/// Updates an existing unit (by name).
///
/// Delegates to the shared `data` crate.
pub fn update_card(conn: &Connection, card: &Unit) -> Result<Option<Unit>> {
    data::cards::unit::update_card(conn, card)
}

/// Renames a unit (updates the primary key `name`) and updates all fields.
///
/// Delegates to the shared `data` crate.
pub fn rename_and_update_card(
    conn: &Connection,
    old_name: &str,
    card: &Unit,
) -> Result<Option<Unit>> {
    data::cards::unit::rename_and_update_card(conn, old_name, card)
}

/// Deletes a unit by name.
///
/// Delegates to the shared `data` crate.
pub fn delete_card(conn: &Connection, name: &str) -> Result<bool> {
    data::cards::unit::delete_card(conn, name)
}

/// Loads a unit by exact name.
///
/// Delegates to the shared `data` crate.
pub fn get_card(conn: &Connection, name: &str) -> Result<Option<Unit>> {
    data::cards::unit::get_card(conn, name)
}
