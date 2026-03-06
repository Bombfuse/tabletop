//! Application state for the simulator.
//!
//! This module intentionally contains *only* state (no DB I/O, no routing/view logic).
//! Side effects belong in `crate::app::handlers`.
//! Rendering/routing belongs in `crate::app::router`.

use crate::db;
use crate::types::Screen;

/// Main `iced` application state.
///
/// Note: the `iced::Application` impl lives in `crate::app::simulator`.
pub struct Simulator {
    pub screen: Screen,

    // Start-campaign flow state
    pub units: Vec<data::cards::unit::Unit>,
    pub selected_hero: Option<String>,
    pub campaign_saved: Option<String>,

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
            Screen::CampaignContinueSelect => "Tabletop Simulator — Continue Campaign".to_string(),
            Screen::CampaignHome { campaign_id } => {
                format!("Tabletop Simulator — Campaign #{campaign_id}")
            }
        }
    }
}
