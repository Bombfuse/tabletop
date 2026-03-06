use iced::{Application, Element, Settings, Theme};

use data::cards::unit;

mod db;
mod pages;
mod types;

const CORE_DB_PATH: &str = "tabletop.sqlite3";

fn main() -> iced::Result {
    // Run simulator migrations on startup (simulator DB is separate from core DB).
    if let Err(e) = db::apply_migrations() {
        eprintln!("Failed to apply simulator migrations: {e:#}");
        // Continue launching UI; DB errors will be surfaced again when starting/continuing a campaign.
    }

    Simulator::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(900.0, 600.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

use types::{Message, Screen};

struct Simulator {
    screen: Screen,

    units: Vec<unit::Unit>,
    selected_hero: Option<String>,
    load_error: Option<String>,

    campaign_saved: Option<String>,

    campaigns: Vec<db::CampaignSummary>,
}

impl Simulator {
    fn title(&self) -> String {
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

impl iced::Application for Simulator {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                screen: Screen::MainMenu,
                units: Vec::new(),
                selected_hero: None,
                load_error: None,
                campaign_saved: None,
                campaigns: Vec::new(),
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        self.title()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::StartCampaign => {
                self.screen = Screen::CampaignSelectHero;

                self.load_error = None;
                self.campaign_saved = None;
                self.units.clear();

                // NOTE: iced `Application::update` is sync, so we load synchronously here.
                // If this becomes slow, move DB calls to a subscription/task.
                //
                // Core tabletop DB is used ONLY for reading reference/unit data.
                let db_path = std::path::Path::new(CORE_DB_PATH);
                match data::db::open_db(db_path).and_then(|conn| unit::list_cards(&conn)) {
                    Ok(units) => self.units = units,
                    Err(e) => self.load_error = Some(e.to_string()),
                }

                iced::Command::none()
            }
            Message::ContinueCampaign => {
                self.screen = Screen::CampaignContinueSelect;

                self.load_error = None;
                self.campaign_saved = None;
                self.campaigns.clear();

                if let Err(e) = db::apply_migrations() {
                    self.load_error = Some(format!("Failed to apply simulator migrations: {e}"));
                    return iced::Command::none();
                }

                match db::open().and_then(|conn| db::list_campaigns(&conn)) {
                    Ok(campaigns) => self.campaigns = campaigns,
                    Err(e) => self.load_error = Some(e.to_string()),
                }

                iced::Command::none()
            }
            Message::SelectHero(name) => {
                self.selected_hero = Some(name);
                self.campaign_saved = None;
                iced::Command::none()
            }
            Message::BeginCampaign => {
                self.load_error = None;
                self.campaign_saved = None;

                let Some(hero_name) = self.selected_hero.clone() else {
                    return iced::Command::none();
                };

                // Ensure simulator DB schema is up-to-date before writing campaign data.
                if let Err(e) = db::apply_migrations() {
                    self.load_error = Some(format!("Failed to apply simulator migrations: {e}"));
                    return iced::Command::none();
                }

                match db::open().and_then(|conn| db::create_campaign(&conn, &hero_name)) {
                    Ok(()) => {
                        self.campaign_saved =
                            Some("Campaign created and saved to simulator database.".to_string());
                    }
                    Err(e) => {
                        self.load_error = Some(e.to_string());
                    }
                }

                iced::Command::none()
            }
            Message::SelectCampaign(campaign_id) => {
                self.load_error = None;
                self.campaign_saved = None;
                self.screen = Screen::CampaignHome { campaign_id };
                iced::Command::none()
            }
            Message::BackToMenu => {
                self.screen = Screen::MainMenu;
                iced::Command::none()
            }
            Message::ExitApp => iced::window::close(iced::window::Id::MAIN),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self.screen {
            Screen::MainMenu => pages::main_menu::view(),
            Screen::CampaignSelectHero => pages::start_campaign::view(
                &self.units,
                self.selected_hero.as_deref(),
                self.campaign_saved.as_deref(),
                self.load_error.as_deref(),
            ),
            Screen::CampaignContinueSelect => {
                pages::continue_campaign::view(&self.campaigns, self.load_error.as_deref())
            }
            Screen::CampaignHome { campaign_id } => pages::campaign_home::view(campaign_id),
        }
    }
}
