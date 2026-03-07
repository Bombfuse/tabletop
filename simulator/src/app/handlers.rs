//! Side-effect handlers for the simulator app.
//!
//! This module contains DB I/O and other side effects (migrations, loads, saves).
//! It is kept separate from:
//! - `crate::app::state` (pure state)
//! - `crate::app::router` (view routing)
//! Side-effect handlers for the simulator app.
//!
//! The handlers mutate `crate::app::state::Simulator` directly to keep `update()`
//! in the `iced::Application` impl small and readable.

use crate::app::state::{HexGridSummary, Simulator};
use crate::{CORE_DB_PATH, db};

/// Loads units from the *core* tabletop database into app state.
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

/// Loads hex grid maps from the *core* tabletop database into app state.
///
/// This is used by the "Load Scenario" screen.
pub fn load_hex_grids_from_core_db(app: &mut Simulator) {
    app.load_error = None;
    app.scenario_hex_grids.clear();
    app.selected_scenario_hex_grid_id = None;

    let db_path = std::path::Path::new(CORE_DB_PATH);
    let result = data::db::open_db(db_path).and_then(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name
            FROM hex_grids
            ORDER BY name ASC
            "#,
        )?;

        let rows = stmt.query_map([], |r| {
            Ok(HexGridSummary {
                id: r.get::<_, i64>(0)?,
                name: r.get::<_, String>(1)?,
            })
        })?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    });

    match result {
        Ok(hex_grids) => {
            app.scenario_hex_grids = hex_grids;
        }
        Err(e) => {
            app.load_error = Some(e.to_string());
        }
    }
}

/// Loads the selected hex grid and its tiles from the *core* tabletop database into app state.
///
/// This is used by the "Scenario Test" view so it can render the grid.
pub fn load_scenario_test_from_core_db(app: &mut Simulator, hex_grid_id: i64) {
    app.load_error = None;
    app.scenario_test_hex_grid = None;
    app.scenario_test_tiles.clear();

    let db_path = std::path::Path::new(CORE_DB_PATH);
    let result = data::db::open_db(db_path).and_then(|conn| {
        let grid = data::hex_grids::HexGrid::load(&conn, hex_grid_id)?;
        let tiles = grid.list_tiles(&conn)?;
        Ok((grid, tiles))
    });

    match result {
        Ok((grid, tiles)) => {
            app.scenario_test_hex_grid = Some(grid);
            app.scenario_test_tiles = tiles;
        }
        Err(e) => {
            app.load_error = Some(e.to_string());
        }
    }
}

/// Ensures the simulator DB schema is up-to-date.
///
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
        Ok(_campaign_id) => {
            app.campaign_saved =
                Some("Campaign created and saved to simulator database.".to_string());
        }
        Err(e) => {
            app.load_error = Some(format!("Failed to create campaign: {e}"));
        }
    }
}
