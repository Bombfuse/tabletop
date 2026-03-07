use iced::alignment::Horizontal;
use iced::widget::{button, column, container, text};
use iced::{Element, Length};

use crate::types::Message;

/// Renders the simulator main menu.
///
/// Routes are handled by the parent `Application` via `Message`s.
pub fn view() -> Element<'static, Message> {
    let title = text("Main Menu")
        .size(44)
        .horizontal_alignment(Horizontal::Center);

    let start = button(text("Start Campaign").size(24))
        .padding(14)
        .width(Length::Fixed(260.0))
        .on_press(Message::StartCampaign);

    let load_scenario = button(text("Load Scenario").size(24))
        .padding(14)
        .width(Length::Fixed(260.0))
        .on_press(Message::LoadScenario);

    let cont = button(text("Continue Campaign").size(24))
        .padding(14)
        .width(Length::Fixed(260.0))
        .on_press(Message::ContinueCampaign);

    let exit = button(text("Exit App").size(24))
        .padding(14)
        .width(Length::Fixed(260.0))
        .on_press(Message::ExitApp);

    let content = column![title, start, load_scenario, cont, exit]
        .spacing(18)
        .align_items(iced::Alignment::Center)
        .width(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding(24)
        .into()
}
