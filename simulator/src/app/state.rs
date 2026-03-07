//! Application state for the simulator.
//!
//! This module intentionally contains *only* state (no DB I/O, no routing/view logic).
//! Side effects belong in `crate::app::handlers`.
//! Rendering/routing belongs in `crate::app::router`.

use crate::db;
use crate::types::Screen;

/// Minimal list row for a hex grid in the core tabletop DB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexGridSummary {
    pub id: i64,
    pub name: String,
}

/// Main `iced` application state.
///
/// Note: the `iced::Application` impl lives in `crate::app::simulator`.
pub struct Simulator {
    pub screen: Screen,

    // Start-campaign flow state
    pub units: Vec<data::cards::unit::Unit>,
    pub selected_hero: Option<String>,
    pub campaign_saved: Option<String>,

    // Load-scenario flow state
    pub scenario_hex_grids: Vec<HexGridSummary>,
    pub selected_scenario_hex_grid_id: Option<i64>,

    // Scenario-test state (loaded hex grid + tiles from the core DB)
    pub scenario_test_hex_grid: Option<data::hex_grids::HexGrid>,
    pub scenario_test_tiles: Vec<data::hex_grids::HexTile>,

    // Continue-campaign flow state
    pub campaigns: Vec<db::CampaignSummary>,

    // Shared error state for DB / loading operations
    pub load_error: Option<String>,
}

impl Simulator {
    /// Creates the initial application state.
    pub fn new() -> Self {
        Self {
            screen: Screen::MainMenu,

            units: Vec::new(),
            selected_hero: None,
            campaign_saved: None,

            scenario_hex_grids: Vec::new(),
            selected_scenario_hex_grid_id: None,

            scenario_test_hex_grid: None,
            scenario_test_tiles: Vec::new(),

            campaigns: Vec::new(),

            load_error: None,
        }
    }

    /// Clears transient UI feedback (errors + "saved" banner).
    pub fn clear_feedback(&mut self) {
        self.load_error = None;
        self.campaign_saved = None;
    }

    /// Resets state for the start-campaign flow.
    pub fn reset_start_campaign_flow(&mut self) {
        self.units.clear();
        self.selected_hero = None;
        self.campaign_saved = None;
        self.load_error = None;
    }

    /// Resets state for the load-scenario flow.
    pub fn reset_load_scenario_flow(&mut self) {
        self.scenario_hex_grids.clear();
        self.selected_scenario_hex_grid_id = None;
        self.load_error = None;
    }

    /// Resets state for the scenario-test view.
    pub fn reset_scenario_test(&mut self) {
        self.scenario_test_hex_grid = None;
        self.scenario_test_tiles.clear();
        self.load_error = None;
    }

    /// Resets state for the continue-campaign flow.
    pub fn reset_continue_campaign_flow(&mut self) {
        self.campaigns.clear();
        self.campaign_saved = None;
        self.load_error = None;
    }

    /// Computes the window title for the current screen.
    pub fn title(&self) -> String {
        match self.screen {
            Screen::MainMenu => "Tabletop Simulator".to_string(),
            Screen::CampaignSelectHero => "Tabletop Simulator — Start Campaign".to_string(),
            Screen::ScenarioSelectHexGrid => "Tabletop Simulator — Load Scenario".to_string(),
            Screen::ScenarioTest { hex_grid_id } => {
                format!("Tabletop Simulator — Scenario Test (Grid #{hex_grid_id})")
            }
            Screen::CampaignContinueSelect => "Tabletop Simulator — Continue Campaign".to_string(),
            Screen::CampaignHome { campaign_id } => {
                format!("Tabletop Simulator — Campaign #{campaign_id}")
            }
        }
    }
}
