use anyhow::Result;
use rusqlite::Connection;

pub use data::cards::interaction::{Interaction, Skill};

/// Lists interactions ordered by action name (ascending).
///
/// Delegates to the shared `data` crate.
pub fn list_cards(conn: &Connection) -> Result<Vec<Interaction>> {
    data::cards::interaction::list_cards(conn)
}

/// Inserts a new interaction for an existing action (by action name).
///
/// Delegates to the shared `data` crate.
pub fn save_card(conn: &Connection, card: &Interaction) -> Result<Interaction> {
    data::cards::interaction::save_card(conn, card)
}

/// Updates an existing interaction (by action name).
///
/// Delegates to the shared `data` crate.
pub fn update_card(conn: &Connection, card: &Interaction) -> Result<Option<Interaction>> {
    data::cards::interaction::update_card(conn, card)
}

/// Moves the interaction association from `old_action_name` to `card.action_name` and updates fields.
///
/// Delegates to the shared `data` crate.
pub fn rename_and_update_card(
    conn: &Connection,
    old_action_name: &str,
    card: &Interaction,
) -> Result<Option<Interaction>> {
    data::cards::interaction::rename_and_update_card(conn, old_action_name, card)
}

/// Deletes an interaction by action name.
///
/// Delegates to the shared `data` crate.
pub fn delete_card(conn: &Connection, action_name: &str) -> Result<bool> {
    data::cards::interaction::delete_card(conn, action_name)
}

/// Loads an interaction by action name.
///
/// Delegates to the shared `data` crate.
pub fn get_card(conn: &Connection, action_name: &str) -> Result<Option<Interaction>> {
    data::cards::interaction::get_card(conn, action_name)
}
