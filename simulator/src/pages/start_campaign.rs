use iced::alignment::Horizontal;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::types::Message;
use data::cards::unit;

/// Renders the "Start Campaign" page:
/// - shows available units (from the core DB; loaded by the app)
/// - lets the user select a hero unit
/// - enables "Begin Campaign" only once a hero is selected
pub fn view(
    units: &[unit::Unit],
    selected_hero: Option<&str>,
    campaign_saved: Option<&str>,
    load_error: Option<&str>,
) -> Element<'static, Message> {
    let heading = text("Start Campaign")
        .size(40)
        .horizontal_alignment(Horizontal::Center);

    let sub = text("Select your hero unit:")
        .size(18)
        .horizontal_alignment(Horizontal::Center);

    let selected = match selected_hero {
        Some(name) => text(format!("Selected hero: {name}"))
            .size(16)
            .horizontal_alignment(Horizontal::Center),
        None => text("Selected hero: (none)")
            .size(16)
            .horizontal_alignment(Horizontal::Center),
    };

    let saved = match campaign_saved {
        Some(msg) => text(msg).size(16).horizontal_alignment(Horizontal::Center),
        None => text("").size(1),
    };

    let error = match load_error {
        Some(e) => text(format!("DB error: {e}"))
            .size(16)
            .horizontal_alignment(Horizontal::Center),
        None => text("").size(1),
    };

    let mut list = column![].spacing(10).width(Length::Fill);

    if units.is_empty() && load_error.is_none() {
        list = list.push(
            text("No units found in the database.")
                .size(16)
                .horizontal_alignment(Horizontal::Center),
        );
    } else {
        for u in units {
            let is_selected = selected_hero == Some(u.name.as_str());

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

    let begin_enabled = selected_hero.is_some();
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
        .center_x()
        .center_y()
        .padding(24)
        .into()
}
