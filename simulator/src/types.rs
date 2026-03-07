//! Shared simulator types.
//!
//! Kept in a dedicated module so UI pages and `main.rs` can share routing and
//! message definitions without circular imports.

/// Top-level navigation states for the simulator UI.
#[derive(Debug, Clone)]
pub enum Screen {
    MainMenu,
    CampaignSelectHero,
    CampaignContinueSelect,

    /// Scenario loading flow: list available hex-grid maps from the core tabletop DB.
    ScenarioSelectHexGrid,

    /// Scenario test view: load + render a specific hex grid (by id) from the core DB.
    ScenarioTest {
        hex_grid_id: i64,
    },

    CampaignHome {
        campaign_id: i64,
    },
}

/// Messages emitted by UI views/pages and handled by the `iced::Application`.
#[derive(Debug, Clone)]
pub enum Message {
    // Main menu actions
    StartCampaign,
    LoadScenario,
    ContinueCampaign,
    ExitApp,

    // Common navigation
    BackToMenu,

    // Start campaign flow
    SelectHero(String),
    BeginCampaign,

    // Load scenario flow
    /// Select a hex grid map by id (from the core tabletop DB list).
    SelectScenarioHexGrid(i64),
    /// Start the scenario using the selected hex grid map.
    StartScenario,

    // Continue campaign flow
    SelectCampaign(i64),
}
