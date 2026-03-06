use anyhow::Result;
use rusqlite::Connection;

pub use data::cards::attack::{Attack, DamageType, Skill};

/// Lists attacks ordered by action name (ascending).
///
/// Delegates to the shared `data` crate.
pub fn list_cards(conn: &Connection) -> Result<Vec<Attack>> {
    data::cards::attack::list_cards(conn)
}

/// Inserts a new attack for an existing action (by action name).
///
/// Delegates to the shared `data` crate.
pub fn save_card(conn: &Connection, card: &Attack) -> Result<Attack> {
    data::cards::attack::save_card(conn, card)
}

/// Updates an existing attack (by action name).
///
/// Delegates to the shared `data` crate.
pub fn update_card(conn: &Connection, card: &Attack) -> Result<Option<Attack>> {
    data::cards::attack::update_card(conn, card)
}

/// Moves the attack association from `old_action_name` to `card.action_name` and updates fields.
///
/// Delegates to the shared `data` crate.
pub fn rename_and_update_card(
    conn: &Connection,
    old_action_name: &str,
    card: &Attack,
) -> Result<Option<Attack>> {
    data::cards::attack::rename_and_update_card(conn, old_action_name, card)
}

/// Deletes an attack by action name.
///
/// Delegates to the shared `data` crate.
pub fn delete_card(conn: &Connection, action_name: &str) -> Result<bool> {
    data::cards::attack::delete_card(conn, action_name)
}

/// Loads an attack by action name.
///
/// Delegates to the shared `data` crate.
pub fn get_card(conn: &Connection, action_name: &str) -> Result<Option<Attack>> {
    data::cards::attack::get_card(conn, action_name)
}
