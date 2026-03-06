use anyhow::Result;
use rusqlite::Connection;

pub use data::cards::level::Level;

/// Lists levels ordered by name (ascending).
///
/// Delegates to the shared `data` crate.
pub fn list_cards(conn: &Connection) -> Result<Vec<Level>> {
    data::cards::level::list_cards(conn)
}

/// Inserts a new level.
///
/// Delegates to the shared `data` crate.
pub fn save_card(conn: &Connection, card: &Level) -> Result<Level> {
    data::cards::level::save_card(conn, card)
}

/// Updates an existing level (by name).
///
/// Delegates to the shared `data` crate.
pub fn update_card(conn: &Connection, card: &Level) -> Result<Option<Level>> {
    data::cards::level::update_card(conn, card)
}

/// Renames a level (updates the primary key `name`) and updates its text.
///
/// Delegates to the shared `data` crate.
pub fn rename_card(conn: &Connection, old_name: &str, card: &Level) -> Result<Option<Level>> {
    data::cards::level::rename_card(conn, old_name, card)
}

/// Deletes a level by name.
///
/// Delegates to the shared `data` crate.
pub fn delete_card(conn: &Connection, name: &str) -> Result<bool> {
    data::cards::level::delete_card(conn, name)
}

/// Loads a level by exact name.
///
/// Delegates to the shared `data` crate.
pub fn get_card(conn: &Connection, name: &str) -> Result<Option<Level>> {
    data::cards::level::get_card(conn, name)
}
