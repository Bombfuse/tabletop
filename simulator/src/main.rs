use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Application, Element, Length, Settings, Theme};

use data::cards::unit;

const CORE_DB_PATH: &str = "tabletop.sqlite3";
const SIMULATOR_DB_PATH: &str = "simulator.sqlite3";

fn main() -> iced::Result {
    Simulator::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(900.0, 600.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
enum Screen {
    MainMenu,
    CampaignSelectHero,
}

#[derive(Debug, Clone)]
enum Message {
    StartCampaign,
    ExitApp,
    BackToMenu,

    SelectHero(String),
    BeginCampaign,
}

struct Simulator {
    screen: Screen,

    units: Vec<unit::Unit>,
    selected_hero: Option<String>,
    load_error: Option<String>,

    campaign_saved: Option<String>,
}

impl Simulator {
    fn title(&self) -> String {
        match self.screen {
            Screen::MainMenu => "Tabletop Simulator".to_string(),
            Screen::CampaignSelectHero => "Tabletop Simulator — Start Campaign".to_string(),
        }
    }

    fn view_main_menu(&self) -> Element<'_, Message> {
        let title = text("Main Menu")
            .size(44)
            .horizontal_alignment(Horizontal::Center);

        let start = button(text("Start Campaign").size(24))
            .padding(14)
            .width(Length::Fixed(260.0))
            .on_press(Message::StartCampaign);

        let exit = button(text("Exit App").size(24))
            .padding(14)
            .width(Length::Fixed(260.0))
            .on_press(Message::ExitApp);

        let content = column![title, start, exit]
            .spacing(18)
            .align_items(iced::Alignment::Center)
            .width(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(24)
            .into()
    }

    fn view_campaign_select_hero(&self) -> Element<'_, Message> {
        let heading = text("Start Campaign")
            .size(40)
            .horizontal_alignment(Horizontal::Center);

        let sub = text("Select your hero unit:")
            .size(18)
            .horizontal_alignment(Horizontal::Center);

        let selected = match self.selected_hero.as_deref() {
            Some(name) => text(format!("Selected hero: {name}"))
                .size(16)
                .horizontal_alignment(Horizontal::Center),
            None => text("Selected hero: (none)")
                .size(16)
                .horizontal_alignment(Horizontal::Center),
        };

        let saved = match self.campaign_saved.as_deref() {
            Some(msg) => text(msg).size(16).horizontal_alignment(Horizontal::Center),
            None => text("").size(1),
        };

        let error = match self.load_error.as_deref() {
            Some(e) => text(format!("DB error: {e}"))
                .size(16)
                .horizontal_alignment(Horizontal::Center),
            None => text("").size(1),
        };

        let mut list = column![].spacing(10).width(Length::Fill);

        if self.units.is_empty() && self.load_error.is_none() {
            list = list.push(
                text("No units found in the database.")
                    .size(16)
                    .horizontal_alignment(Horizontal::Center),
            );
        } else {
            for u in &self.units {
                let is_selected = self.selected_hero.as_deref() == Some(u.name.as_str());

                let name = if is_selected {
                    format!("{} (Hero)", u.name)
                } else {
                    u.name.clone()
                };

                let stats = format!(
                    "STR {}  FOC {}  INT {}  AGI {}  KNO {}",
                    u.strength, u.focus, u.intelligence, u.agility, u.knowledge
                );

                let select = button(text("Select").size(14))
                    .padding(8)
                    .on_press(Message::SelectHero(u.name.clone()));

                let row_item = row![
                    column![text(name).size(18), text(stats).size(14)].spacing(4),
                    select
                ]
                .spacing(16)
                .align_items(iced::Alignment::Center);

                list = list.push(container(row_item).padding(10));
            }
        }

        let list = scrollable(container(list).width(Length::Fill)).height(Length::FillPortion(1));

        let begin_enabled = self.selected_hero.is_some();
        let mut begin = button(text("Begin Campaign").size(18))
            .padding(12)
            .width(Length::Fixed(200.0));
        if begin_enabled {
            begin = begin.on_press(Message::BeginCampaign);
        }

        let back = button(text("Back to Menu").size(18))
            .padding(12)
            .width(Length::Fixed(200.0))
            .on_press(Message::BackToMenu);

        let content = column![heading, sub, selected, saved, error, list, begin, back]
            .spacing(14)
            .align_items(iced::Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(24)
            .into()
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

                // Simulator DB is separate from the core tabletop DB and is used for campaign persistence.
                // We keep it simple and create the schema on demand.
                match rusqlite::Connection::open(SIMULATOR_DB_PATH)
                    .map_err(anyhow::Error::from)
                    .and_then(|conn| {
                        conn.pragma_update(None, "foreign_keys", "ON")?;
                        conn.pragma_update(None, "journal_mode", "WAL")?;

                        conn.execute_batch(
                            r#"
                            CREATE TABLE IF NOT EXISTS campaigns (
                                id                INTEGER PRIMARY KEY AUTOINCREMENT,
                                hero_unit_name     TEXT NOT NULL,
                                created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                            );
                            CREATE INDEX IF NOT EXISTS idx_campaigns_created_at ON campaigns(created_at);
                            "#,
                        )?;

                        conn.execute(
                            r#"
                            INSERT INTO campaigns (hero_unit_name)
                            VALUES (?1)
                            "#,
                            rusqlite::params![hero_name],
                        )?;

                        Ok(())
                    }) {
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
            Message::BackToMenu => {
                self.screen = Screen::MainMenu;
                iced::Command::none()
            }
            Message::ExitApp => iced::window::close(iced::window::Id::MAIN),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self.screen {
            Screen::MainMenu => self.view_main_menu(),
            Screen::CampaignSelectHero => self.view_campaign_select_hero(),
        }
    }
}
