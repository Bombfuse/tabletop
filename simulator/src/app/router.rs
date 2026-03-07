//! View router for the simulator app.
//!
//! This module maps high-level navigation state (`Screen`) to a concrete UI
//! view function in `crate::pages`.
//!
//! Keeping this separate from:
//! - `crate::app::state` (pure state)
//! - `crate::app::handlers` (side effects / DB I/O)
//! - `crate::app::simulator` (iced::Application glue)
//! makes the app easier to extend as more screens are added.

use iced::Element;

use crate::app::state::Simulator;
use crate::types::{Message, Screen};

/// Render the appropriate view for the current screen.
///
/// Note: Views are intentionally "dumb": they should render based on the state
/// provided and emit `Message`s for the `iced::Application` to handle.
pub fn view(app: &Simulator) -> Element<'_, Message> {
    match app.screen {
        Screen::MainMenu => crate::pages::main_menu::view(),
        Screen::CampaignSelectHero => crate::pages::start_campaign::view(
            &app.units,
            app.selected_hero.as_deref(),
            app.campaign_saved.as_deref(),
            app.load_error.as_deref(),
        ),
        Screen::ScenarioSelectHexGrid => crate::pages::load_scenario::view(
            &app.scenario_hex_grids,
            app.selected_scenario_hex_grid_id,
            app.load_error.as_deref(),
        ),
        Screen::ScenarioTest { hex_grid_id } => crate::pages::scenario_test::view(
            hex_grid_id,
            app.scenario_test_hex_grid.as_ref(),
            &app.scenario_test_tiles,
            app.load_error.as_deref(),
        ),
        Screen::CampaignContinueSelect => {
            crate::pages::continue_campaign::view(&app.campaigns, app.load_error.as_deref())
        }
        Screen::CampaignHome { campaign_id } => crate::pages::campaign_home::view(campaign_id),
    }
}
