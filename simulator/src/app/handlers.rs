//! Side-effect handlers for the simulator app.
//!
//! This module contains DB I/O and other side effects (migrations, loads, saves).
//! It is kept separate from:
//! - `crate::app::state` (pure state)
//! - `crate::app::router` (view routing)
//!
//! The handlers mutate `crate::app::state::Simulator` directly to keep `update()`
//! in the `iced::Application` impl small and readable.

use crate::app::state::Simulator;
use crate::{CORE_DB_PATH, db};

/// Loads units from the *core* tabletop database into app state.
///
/// - Uses the core DB strictly as read-only reference data.
/// - Populates `app.units` on success.
/// - Sets `app.load_error` on failure.
pub fn load_units_from_core_db(app: &mut Simulator) {
    app.load_error = None;
    app.units.clear();

    let db_path = std::path::Path::new(CORE_DB_PATH);
    match data::db::open_db(db_path).and_then(|conn| data::cards::unit::list_cards(&conn)) {
        Ok(units) => {
            app.units = units;
        }
        Err(e) => {
            app.load_error = Some(e.to_string());
        }
    }
}

/// Ensures the simulator DB schema is up-to-date.
///
/// On failure, sets `app.load_error` and returns `false`.
pub fn ensure_simulator_db_ready(app: &mut Simulator) -> bool {
    match db::apply_migrations() {
        Ok(()) => true,
        Err(e) => {
            app.load_error = Some(format!("Failed to apply simulator migrations: {e}"));
            false
        }
    }
}

/// Loads existing campaigns from the simulator database into app state.
///
/// - Ensures migrations are applied before reading.
/// - Populates `app.campaigns` on success.
/// - Sets `app.load_error` on failure.
pub fn load_campaigns_from_simulator_db(app: &mut Simulator) {
    app.load_error = None;
    app.campaigns.clear();

    if !ensure_simulator_db_ready(app) {
        return;
    }

    match db::open().and_then(|conn| db::list_campaigns(&conn)) {
        Ok(campaigns) => {
            app.campaigns = campaigns;
        }
        Err(e) => {
            app.load_error = Some(e.to_string());
        }
    }
}

/// Creates a new campaign in the simulator database using the currently-selected hero.
///
/// Requirements:
/// - `app.selected_hero` must be `Some`.
///
/// Effects:
/// - Ensures migrations are applied before writing.
/// - Sets `app.campaign_saved` on success.
/// - Sets `app.load_error` on failure.
pub fn create_campaign_from_selected_hero(app: &mut Simulator) {
    app.load_error = None;
    app.campaign_saved = None;

    // Clone hero name up front so we don't hold an immutable borrow of `app`
    // across calls that need `&mut app`.
    let hero_name = match app.selected_hero.as_deref() {
        Some(name) => name.to_string(),
        None => {
            // UI should keep the button disabled, but we guard anyway.
            app.load_error = Some("No hero selected.".to_string());
            return;
        }
    };

    if !ensure_simulator_db_ready(app) {
        return;
    }

    match db::open().and_then(|conn| db::create_campaign(&conn, &hero_name)) {
        Ok(()) => {
            app.campaign_saved =
                Some("Campaign created and saved to simulator database.".to_string());
        }
        Err(e) => {
            app.load_error = Some(e.to_string());
        }
    }
}
