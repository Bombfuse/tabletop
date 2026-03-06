use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Application, Element, Length, Settings, Theme};

use data::cards::unit;

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
}

struct Simulator {
    screen: Screen,

    units: Vec<unit::Unit>,
    selected_hero: Option<String>,
    load_error: Option<String>,
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

        let back = button(text("Back to Menu").size(18))
            .padding(12)
            .width(Length::Fixed(200.0))
            .on_press(Message::BackToMenu);

        let content = column![heading, sub, selected, error, list, back]
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
                self.units.clear();

                // NOTE: iced `Application::update` is sync, so we load synchronously here.
                // If this becomes slow, move DB calls to a subscription/task.
                let db_path = std::path::Path::new("tabletop.sqlite3");
                match data::db::open_db(db_path).and_then(|conn| unit::list_cards(&conn)) {
                    Ok(units) => self.units = units,
                    Err(e) => self.load_error = Some(e.to_string()),
                }

                iced::Command::none()
            }
            Message::SelectHero(name) => {
                self.selected_hero = Some(name);
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
