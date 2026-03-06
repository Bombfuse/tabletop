use anyhow::Result;
use rusqlite::Connection;

pub use data::cards::action::{
    Action, ActionAssociation, ActionType, clear_association, get_association, set_association,
};

/// Lists actions ordered by name (ascending).
///
/// Delegates to the shared `data` crate.
pub fn list_cards(conn: &Connection) -> Result<Vec<Action>> {
    data::cards::action::list_cards(conn)
}

/// Inserts a new action.
///
/// Delegates to the shared `data` crate.
pub fn save_card(conn: &Connection, card: &Action) -> Result<Action> {
    data::cards::action::save_card(conn, card)
}

/// Updates an existing action (by name).
///
/// Delegates to the shared `data` crate.
pub fn update_card(conn: &Connection, card: &Action) -> Result<Option<Action>> {
    data::cards::action::update_card(conn, card)
}

/// Renames an action (updates the primary key `name`) and updates all fields.
///
/// Delegates to the shared `data` crate.
pub fn rename_and_update_card(
    conn: &Connection,
    old_name: &str,
    card: &Action,
) -> Result<Option<Action>> {
    data::cards::action::rename_and_update_card(conn, old_name, card)
}

/// Deletes an action by name.
///
/// Delegates to the shared `data` crate.
pub fn delete_card(conn: &Connection, name: &str) -> Result<bool> {
    data::cards::action::delete_card(conn, name)
}

/// Loads an action by exact name.
///
/// Delegates to the shared `data` crate.
pub fn get_card(conn: &Connection, name: &str) -> Result<Option<Action>> {
    data::cards::action::get_card(conn, name)
}

// NOTE: Tests that validate `data::cards::action` behavior (including cascade deletes to
// `attacks` / `interactions`) should live in the `data` crate alongside the implementation,
// using `data::cards::test_support` for in-memory DB + schema setup.
//
// This `tools` crate module is a thin delegation layer, so keeping duplicated schema creation
// logic here is unnecessary.
