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
    CampaignHome { campaign_id: i64 },
}

/// Messages emitted by UI views/pages and handled by the `iced::Application`.
#[derive(Debug, Clone)]
pub enum Message {
    // Main menu actions
    StartCampaign,
    ContinueCampaign,
    ExitApp,

    // Common navigation
    BackToMenu,

    // Start campaign flow
    SelectHero(String),
    BeginCampaign,

    // Continue campaign flow
    SelectCampaign(i64),
}
