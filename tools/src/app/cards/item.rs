use anyhow::Result;
use rusqlite::Connection;

pub use data::cards::item::Item;

/// Lists items ordered by name (ascending).
///
/// Delegates to the shared `data` crate.
pub fn list_cards(conn: &Connection) -> Result<Vec<Item>> {
    data::cards::item::list_cards(conn)
}

/// Inserts a new item.
///
/// Delegates to the shared `data` crate.
pub fn save_card(conn: &Connection, card: &Item) -> Result<Item> {
    data::cards::item::save_card(conn, card)
}

/// Updates an existing item (by name).
///
/// Delegates to the shared `data` crate.
pub fn update_card(conn: &Connection, card: &Item) -> Result<Option<Item>> {
    data::cards::item::update_card(conn, card)
}

/// Renames an item (updates the primary key `name`).
///
/// Delegates to the shared `data` crate.
pub fn rename_card(conn: &Connection, old_name: &str, card: &Item) -> Result<Option<Item>> {
    data::cards::item::rename_card(conn, old_name, card)
}

/// Deletes an item by name.
///
/// Delegates to the shared `data` crate.
pub fn delete_card(conn: &Connection, name: &str) -> Result<bool> {
    data::cards::item::delete_card(conn, name)
}

/// Loads an item by exact name.
///
/// Delegates to the shared `data` crate.
pub fn get_card(conn: &Connection, name: &str) -> Result<Option<Item>> {
    data::cards::item::get_card(conn, name)
}
