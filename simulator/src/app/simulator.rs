//! `iced::Application` glue for the simulator.
//!
//! This module ties together:
//! - `crate::app::state` (pure state)
//! - `crate::app::handlers` (side effects / DB I/O)
//! - `crate::app::router` (Screen -> View mapping)

use iced::{Application, Element, Theme};

use crate::app::{handlers, router};
use crate::types::Message;

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
                self.screen = crate::types::Screen::CampaignSelectHero;
                self.reset_start_campaign_flow();
                handlers::load_units_from_core_db(self);
                iced::Command::none()
            }
            Message::ContinueCampaign => {
                self.screen = crate::types::Screen::CampaignContinueSelect;
                self.reset_continue_campaign_flow();
                handlers::load_campaigns_from_simulator_db(self);
                iced::Command::none()
            }
            Message::ExitApp => iced::window::close(iced::window::Id::MAIN),

            // Common navigation
            Message::BackToMenu => {
                self.screen = crate::types::Screen::MainMenu;
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

            // Continue campaign flow
            Message::SelectCampaign(campaign_id) => {
                self.screen = crate::types::Screen::CampaignHome { campaign_id };
                self.clear_feedback();
                iced::Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        router::view(self)
    }
}
