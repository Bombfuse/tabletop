use iced::alignment::Horizontal;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::app::state::HexGridSummary;
use crate::types::Message;

/// Load Scenario screen.
///
/// Shows a list of Hex Grid maps (from the core tabletop SQLite DB).
/// Each row displays the grid `Name` and a `Start` button.
pub fn view(
    hex_grids: &[HexGridSummary],
    _selected_hex_grid_id: Option<i64>,
    load_error: Option<&str>,
) -> Element<'static, Message> {
    let title = text("Load Scenario")
        .size(44)
        .horizontal_alignment(Horizontal::Center);

    let mut content = column![title].spacing(18).width(Length::Fill);

    if let Some(err) = load_error {
        content = content.push(text(err).size(16).style(iced::theme::Text::Color(
            iced::Color::from_rgb(0.85, 0.2, 0.2),
        )));
    }

    // Header row
    let header = row![
        text("Name")
            .size(18)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Left),
        text("") // spacer for button column
            .size(18)
            .width(Length::Fixed(120.0))
    ]
    .spacing(12)
    .align_items(iced::Alignment::Center);

    let mut list = column![header].spacing(10).width(Length::Fill);

    if hex_grids.is_empty() {
        list = list.push(text("No hex grid maps found.").size(18));
    } else {
        for grid in hex_grids {
            let start_btn = button(text("Start").size(16))
                .padding(10)
                .width(Length::Fixed(120.0))
                // For now, starting a scenario is modelled as selecting the grid.
                // The app can later handle `Message::SelectScenarioHexGrid(grid.id)`
                // by loading/starting the scenario.
                .on_press(Message::SelectScenarioHexGrid(grid.id));

            let r = row![
                text(&grid.name)
                    .size(18)
                    .width(Length::Fill)
                    .horizontal_alignment(Horizontal::Left),
                start_btn
            ]
            .spacing(12)
            .align_items(iced::Alignment::Center);

            list = list.push(r);
        }
    }

    let scroll = scrollable(list).height(Length::Fill);

    let back = button(text("Back").size(18))
        .padding(12)
        .width(Length::Fixed(160.0))
        .on_press(Message::BackToMenu);

    content = content
        .push(scroll)
        .push(row![back].width(Length::Fill).spacing(12));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .into()
}
