//! `iced::Application` glue for the simulator.
//!
//! This module ties together:
//! - `crate::app::state` (pure state)
//! - `crate::app::handlers` (side effects / DB I/O)
//! - `crate::app::router` (Screen -> View mapping)
//!
//! It also wires top-level navigation `Message`s into transitions between `Screen`s.

use iced::{Application, Element, Theme};

use crate::app::{handlers, router};
use crate::types::{Message, Screen};

pub use crate::app::state::Simulator;

impl Application for Simulator {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self::new(), iced::Command::none())
    }

    fn title(&self) -> String {
        self.title()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            // Main menu actions
            Message::StartCampaign => {
                self.screen = Screen::CampaignSelectHero;
                self.reset_start_campaign_flow();
                handlers::load_units_from_core_db(self);
                iced::Command::none()
            }
            Message::LoadScenario => {
                self.screen = Screen::ScenarioSelectHexGrid;
                self.reset_load_scenario_flow();
                handlers::load_hex_grids_from_core_db(self);
                iced::Command::none()
            }
            Message::ContinueCampaign => {
                self.screen = Screen::CampaignContinueSelect;
                self.reset_continue_campaign_flow();
                handlers::load_campaigns_from_simulator_db(self);
                iced::Command::none()
            }
            Message::ExitApp => iced::window::close(iced::window::Id::MAIN),

            // Common navigation
            Message::BackToMenu => {
                self.screen = Screen::MainMenu;
                self.clear_feedback();
                iced::Command::none()
            }

            // Start campaign flow
            Message::SelectHero(name) => {
                self.selected_hero = Some(name);
                self.clear_feedback();
                iced::Command::none()
            }
            Message::BeginCampaign => {
                handlers::create_campaign_from_selected_hero(self);
                iced::Command::none()
            }

            // Load scenario flow
            Message::SelectScenarioHexGrid(hex_grid_id) => {
                // "Start" from the scenario list: navigate to Scenario Test and load + render the grid.
                self.screen = Screen::ScenarioTest { hex_grid_id };
                self.reset_scenario_test();
                handlers::load_scenario_test_from_core_db(self, hex_grid_id);
                iced::Command::none()
            }
            Message::StartScenario => {
                // If a future UI adds a separate "Start Scenario" button, use the selected id.
                let Some(hex_grid_id) = self.selected_scenario_hex_grid_id else {
                    self.load_error = Some("No scenario hex grid selected.".to_string());
                    return iced::Command::none();
                };

                self.screen = Screen::ScenarioTest { hex_grid_id };
                self.reset_scenario_test();
                handlers::load_scenario_test_from_core_db(self, hex_grid_id);
                iced::Command::none()
            }

            // Continue campaign flow
            Message::SelectCampaign(campaign_id) => {
                self.screen = Screen::CampaignHome { campaign_id };
                self.clear_feedback();
                iced::Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        router::view(self)
    }
}
